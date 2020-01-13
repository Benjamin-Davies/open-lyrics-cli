use clap::*;

mod cmd;

fn main() {
    let app = App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .version(crate_version!())
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .subcommand(cmd::bible::make_subcommand());

    let db_dir: String = format!("{}/.local/share/openlp", std::env::var("HOME").unwrap());

    match app.get_matches().subcommand() {
        ("bible", Some(sub_matches)) => cmd::bible::execute(&db_dir, sub_matches),
        (_, _) => unreachable!(),
    }
}
