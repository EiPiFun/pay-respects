use crate::shell::Data;
use crate::suggestions::{best_match_path, suggest_candidates};
use crate::system;
use crate::{shell, suggestions};
use colored::Colorize;
use inquire::*;

pub fn suggestion(data: &mut Data) {
	let shell = data.shell.clone();
	let mut last_command;

	loop {
		last_command = data.command.clone();
		suggest_candidates(data);
		if data.candidates.is_empty() {
			break;
		};

		for candidate in &mut data.candidates {
			shell::shell_syntax(&shell, candidate);
		}

		suggestions::select_candidate(data);

		let execution = suggestions::confirm_suggestion(data);
		if execution.is_ok() {
			return;
		} else {
			data.update_command(&data.suggest.clone().unwrap());
			let msg = Some(
				execution
					.err()
					.unwrap()
					.split_whitespace()
					.collect::<Vec<&str>>()
					.join(" "),
			);
			data.update_error(msg);

			let retry_message = format!("{}...", t!("retry"));

			eprintln!("\n{}\n", retry_message.cyan().bold());
		}
	}
	eprintln!("{}: {}\n", t!("no-suggestion"), last_command.red());
	eprintln!(
		"{}\n{}",
		t!("contribute"),
		option_env!("CARGO_PKG_REPOSITORY").unwrap_or("https://github.com/iffse/pay-respects/")
	);
}

pub fn cnf(data: &mut Data) {
	let shell = data.shell.clone();
	let mut split_command = data.split.clone();

	let executable = split_command[0].as_str();
	eprintln!(
		"{}: {}: {}\n",
		shell.red(),
		t!("command-not-found"),
		executable
	);

	let best_match = best_match_path(executable);
	if best_match.is_some() {
		let best_match = best_match.unwrap();
		split_command[0] = best_match;
		let suggest = split_command.join(" ");

		data.candidates.push(suggest.clone());
		suggestions::select_candidate(data);

		let status = suggestions::confirm_suggestion(data);
		if status.is_err() {
			data.update_command(&suggest);
			let msg = Some(
				status
					.err()
					.unwrap()
					.split_whitespace()
					.collect::<Vec<&str>>()
					.join(" "),
			);
			data.update_error(msg);
		}
		suggestion(data);
	} else {
		let package_manager = match system::get_package_manager(&shell) {
			Some(package_manager) => package_manager,
			None => {
				return;
			}
		};

		let packages = match system::get_packages(&shell, &package_manager, executable)
		{
			Some(packages) => packages,
			None => {
				eprintln!(
					"{}: {}",
					"pay-respects".red(),
					t!("package-not-found")
				);
				return;
			}
		};

		let style = ui::Styled::default();
		let render_config = ui::RenderConfig::default()
			.with_prompt_prefix(style);
		let msg = format!("{}", t!("install-package"))
			.bold()
			.blue();
		let hint = format!(
			"{} {} {}",
			"[↑/↓]".blue(),
			t!("confirm-yes").green(),
			"[Ctrl+C]".red()
		);
		eprintln!("{}", msg);
		eprintln!("{}", hint);
		let package = Select::new("\n", packages)
			.without_help_message()
			.with_render_config(render_config)
			.without_filtering()
			.prompt().unwrap();

		// retry after installing package
		if system::install_package(&shell, &package_manager, &package) {
			let _ = suggestions::run_suggestion(data, &data.command);
		}
	}
}
