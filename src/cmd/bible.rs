use clap::*;
use rusqlite::*;
use std::ops::Range;
use std::path::Path;

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
        .subcommand(App::new("verses")
            .arg_from_usage("<book>")
            .arg_from_usage("<chapters>")
            .arg_from_usage("[verses]"))
}

pub fn execute(db_dir: &str, matches: &ArgMatches) {
    let version = matches.value_of("version")
        .unwrap_or("KJV");
    let db_path = format!("{}/bibles/{}.sqlite", db_dir, version);
    let db_path = Path::new(&db_path);
    if !db_path.exists() {
        panic!("Version {} is not installed", version);
    }

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
        ("verses", Some(sub_matches)) => {
            let book = sub_matches.value_of("book").unwrap();
            let chapters = sub_matches.value_of("chapters").map(parse_range).unwrap();
            let verses = sub_matches.value_of("verses").map(parse_ranges).unwrap_or_default();

            let book_id = get_book_id(&conn, book);

            // Dynamically generated query
            // Not vulnerable to SQL injection as all variables used are restricted to
            // integers
            let mut query = String::from("SELECT chapter, verse, text FROM verse");
            query.push_str(&format!(" WHERE book_id = {} AND ", book_id));
            query.push_str(&range_sql_for_prop(chapters, "chapter"));
            if verses.len() > 0 {
                query.push_str(" AND (");
                query.push_str(&verses
                    .into_iter()
                    .map(|r| range_sql_for_prop(r, "verse"))
                    .collect::<Vec<String>>()
                    .join(" OR "));
                query.push_str(")");
            }

            eprintln!("{}", query);
            let mut stmt = conn.prepare(&query).unwrap();
            let verse_iter = stmt
                .query_map(
                    params![],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap();

            for tuple in verse_iter {
                let (chapter, verse, text): (i64, i64, String) = tuple.unwrap();
                println!("{}:{} {}", chapter, verse, text);
            }
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

fn parse_ranges(s: &str) -> Vec<Range<i64>> {
    s.split(",").map(parse_range).collect()
}

fn parse_range(s: &str) -> Range<i64> {
    let parts: Box<[&str]> = s.split("-").collect();
    match parts.len() {
        1 => {
            let a = parts[0].parse().unwrap();
            a..a
        }
        2 => {
            let a = parts[0].parse().unwrap();
            let b = parts[1].parse().unwrap();
            a..b
        }
        _ => {
            panic!("Can't parse range: {}", s);
        }
    }
}

fn range_sql_for_prop(r: Range<i64>, prop: &str) -> String {
    format!(
        "({} >= {} AND {} <= {})",
        prop, r.start,
        prop, r.end)
}
