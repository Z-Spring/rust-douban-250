#![allow(unused)]

use futures::future::ok;
use reqwest::{Client, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use std::error::Error;
use std::future::Future;
use std::io::{Read, Write};
use std::string::ParseError;
use std::{io, result};
use tokio::fs::File;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::time::Instant;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Movie {
    id: usize,
    name: String,
    img: String,
    actor: String,
    time: String,
}

pub async fn fetch_html_infos(start_page: u8) -> reqwest::Result<String> {
    let uri = format!(
        "https://movie.douban.com/top250?start={}&filter=",
        start_page
    );
    let client = Client::new();
    let resp = client
        .get(uri)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36")
        .header("Cookie", r#"ll="108288"; bid=tfmu0VNtswM; douban-fav-remind=1; push_noty_num=0; push_doumail_num=0; viewed="1426971_27192353_35933934_25782902_35196328_35960106_26979890_30357170_30397714_27663285"; dbcl2="156810236:Lnq31Eo+jXg"; ck=u6u5"#)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}

/* pub async fn get_movie_infos(start_page: u8) -> Vec<Movie> {
    let mut resp = match fetch_html_infos(start_page).await {
        Ok(resp) => resp,
        Err(err) => format!("{}", err),
    };

    let doc = Html::parse_document(&resp);
    let selector = Selector::parse("div.item").unwrap();

    let items = doc.select(&selector).collect::<Vec<_>>();
    let mut movies: Vec<Movie> = vec![];

    // println!("{}", items.len());
    for i in 0..items.len() {
        let title_selector = Selector::parse("span.title").expect("parse title error");
        let id_selector = Selector::parse("em").unwrap();
        let img_selector = Selector::parse(r#"a > img[width = "100"]"#).unwrap();
        let time_and_actor_selector = Selector::parse("div.bd > p").unwrap();

        let  time_and_actor = &items[i]
            .select(&time_and_actor_selector)
            .next()
            .unwrap()
            .inner_html();

        let movie_time = time_and_actor.split("\n").collect::<Vec<_>>()[2];
        let movie_actor = time_and_actor.trim_start().split("\n").collect::<Vec<_>>()[0]
            .trim_end_matches("...<br>");

        let movie_time = match movie_time.trim_start().split_once("&nbsp") {
            Some(t) => t.0,
            None => "",
        };

        let img = &items[i]
            .select(&img_selector)
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap();

        let id = &items[i]
            .select(&id_selector)
            .next()
            .unwrap()
            .inner_html()
            .parse::<usize>()
            .unwrap();

        let name = items[i]
            .select(&title_selector)
            .next()
            .unwrap()
            .inner_html();

        let movie = Movie {
            id: *id,
            name: name.to_string(),
            actor: movie_actor.to_string(),
            time: movie_time.to_string(),
            img: img.to_string(),
        };
        movies.push(movie);
    }
    movies
} */

pub async fn get_movie_infos(start_page: u8) -> Vec<Movie> {
    let mut resp = match fetch_html_infos(start_page).await {
        Ok(resp) => resp,
        Err(err) => format!("{}", err),
    };

    let doc = Html::parse_document(&resp);
    let soup = Soup::new(&resp);
    let mut movies: Vec<Movie> = vec![];

    let item = soup.attr_value("item").find_all().collect::<Vec<_>>();
    for i in 0..item.len() {
        let id = &item[i]
            .tag("em")
            .find()
            .expect("can't find this em tag!")
            .text()
            .parse::<usize>()
            .expect("can't parse from String to usize");

        let img = &item[i]
            .tag("img")
            // .attr_name("src")
            .find()
            .expect("can't find img tag!")
            .get("src")
            .expect("img is None!");

        let name = &item[i]
            .tag("span")
            .find()
            .expect("can't find span tag!")
            .text();

        let actor_and_time = &item[i]
            .tag("p")
            .find()
            .expect("can't find the p tag!")
            .text();
        let movie_time = actor_and_time.split("\n").collect::<Vec<_>>()[2];
        let movie_actor = actor_and_time.trim_start().split("\n").collect::<Vec<_>>()[0]
            .trim_end_matches("...<br>");

        let movie_time = match movie_time.trim_start().split_once("\u{a0}") {
            Some(t) => t.0,
            None => "",
        };

        let movie = Movie {
            id: *id,
            name: name.to_string(),
            actor: movie_actor.to_string(),
            time: movie_time.to_string(),
            img: img.to_string(),
        };
        movies.push(movie);
        // println!("{:#?}", movie);
    }
    movies
}

pub fn struct_to_json(movie: Vec<Movie>) -> serde_json::Result<String> {
    let m = serde_json::to_string_pretty(&movie)?;
    Ok(m)
}

pub async fn write_to_file(movie: Vec<Movie>) -> result::Result<(), io::Error> {
    let mut data = match struct_to_json(movie) {
        Ok(m) => format!("```json\n{}\n```", m),
        Err(errs) => format!("{}", errs),
    };
    let mut file = File::create("README.md").await?;
    //    file.write(b )
    file.write_all(b"> use Rust to crawl douban moive infos.\n")
        .await?;
    file.write_all(&data.as_bytes()).await?;
    // file.write_all(&data.as_bytes())?;
    Ok(())
}

#[test]
fn test_struct_to_json() {
    let mut movie: Vec<Movie> = vec![];
    let movie1 = Movie {
        id: 1,
        name: "机器人总动员".to_string(),
        img: "https://img2.doubanio.com/view/photo/s_ratio_poster/public/p1461851991.jpg"
            .to_string(),
        actor: "".to_string(),
        time: "".to_string(),
    };
    let movie2 = Movie {
        id: 3,
        name: "三傻大闹宝莱坞".to_string(),
        img: "https://img2.doubanio.com/view/photo/s_ratio_poster/public/p579729551.jpg"
            .to_string(),
        actor: "".to_string(),
        time: "".to_string(),
    };

    movie.push(movie1);
    movie.push(movie2);
    let result = struct_to_json(movie);
    match result {
        Ok(m) => println!("{}", m),
        Err(err) => println!("{}", err),
    }
}

#[test]
fn test_get_movie_name() {
    /* let mut file = File::create("README.md").await.unwrap();
    file.write_all("hello".as_bytes()).await.unwrap(); */
    let mut file = std::fs::File::open("movie.html").unwrap();
    let mut resp = String::new();
    file.read_to_string(&mut resp);
    let doc = Html::parse_document(&resp);

    // let movies = get_movie_names(&doc);
    // println!("{:#?}",movies);
}
