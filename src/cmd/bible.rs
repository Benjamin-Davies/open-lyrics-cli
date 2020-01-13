use clap::*;
use rusqlite::*;

pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    App::new("bible")
        .about("Read bible verses")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg_from_usage("-v --version=[version] 'Version to use (defualt KJV)'")
        .subcommand(App::new("books"))
        .subcommand(App::new("verse")
            .arg_from_usage("<book>")
            .arg_from_usage("<chapter>")
            .arg_from_usage("<verse>"))
}

pub fn execute(db_dir: &str, matches: &ArgMatches) {
    let version = matches.value_of("version")
        .unwrap_or("KJV");

    let db_path = format!("{}/bibles/{}.sqlite", db_dir, version);
    let conn = Connection::open(db_path).unwrap();

    match matches.subcommand() {
        ("books", Some(_)) => {
            let mut stmt = conn.prepare("SELECT name FROM book ORDER BY book_reference_id ASC").unwrap();
            let book_iter = stmt.query_map(params![], |r| r.get(0)).unwrap();

            for book in book_iter {
                let book: String = book.unwrap();
                println!("{}", book);
            }
        }
        ("verse", Some(sub_matches)) => {
            let book = sub_matches.value_of("book").unwrap();
            let chapter: i64 = sub_matches.value_of("chapter").unwrap().parse().unwrap();
            let verse: i64 = sub_matches.value_of("verse").unwrap().parse().unwrap();

            let book_id = get_book_id(&conn, book);

            let text = conn
                .query_row(
                    "SELECT text FROM verse
                        WHERE book_id = ? AND chapter = ? AND verse = ?",
                    params![
                        book_id,
                        chapter,
                        verse,
                    ],
                    |r| r.get(0),
                );

            let text: String = text.unwrap();
            println!("{}", text);
        }
        (_, _) => { unreachable!(); }
    }
}

fn get_book_id(conn: &Connection, name: &str) -> i64 {
    conn
        .query_row(
            "SELECT id FROM book WHERE name = ?",
            &[name],
            |r| r.get(0),
        )
        .unwrap()
}
