#![allow(unused)]

use reqwest::{Client, Result};
use scraper::{Html, Selector};
use std::error::Error;
use std::future::Future;
use std::result;
use tokio::io::split;

pub mod movie;

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct MovieInfo {
        name: String,
        img: String,
    }

    #[test]
    fn get_movie_name() {
        let mut file = File::open("movie.html").unwrap();
        let mut html = String::new();
        file.read_to_string(&mut html).unwrap();

        let doc = Html::parse_document(&html);
        let title_selector = Selector::parse("span.title").unwrap();
        let id_selector = Selector::parse("em").unwrap();
        let img_selector = Selector::parse(r#"a > img[width = "100"]"#).unwrap();
        let time_selector = Selector::parse("div.bd > p").unwrap();
        // let quote

        let mut times = doc
            .select(&time_selector)
            .map(|x| x.inner_html())
            .filter(|x| x.contains("&nbsp"))
            .collect::<Vec<_>>();

        let mut release_times: Vec<&str> = vec![];
        let mut movie_actors: Vec<&str> = vec![];
        for time in &times {
            let movie_time = time.split("\n").collect::<Vec<_>>()[2];
            let movie_actor =
                time.trim_start().split("\n").collect::<Vec<_>>()[0].trim_end_matches("...<br>");

            let movie_time = match movie_time.trim_start().split_once("&nbsp") {
                Some(t) => t.0,
                None => "",
            };
            release_times.push(movie_time);
            movie_actors.push(movie_actor);
        }

        // let ids = doc
        //     .select(&id_selector)
        //     .next()
        //     .unwrap()
        //     .text()
        //     .collect::<Vec<_>>();
        let mut movie_ids = vec![];
        let ids = doc
            .select(&id_selector)
            .map(|x| x.inner_html())
            .for_each(|x| {
                let p = x.parse::<u32>().unwrap();
                movie_ids.push(p);
            });

        let titles = doc
            .select(&title_selector)
            .map(|x| x.inner_html())
            .filter(|x| !x.starts_with("&nbsp"))
            .collect::<Vec<_>>();

        let imgs = doc
            .select(&img_selector)
            .map(|x| x.value().attr("src").unwrap())
            .collect::<Vec<&str>>();

        let mut movies: Vec<MovieInfo> = vec![];

        let name = &titles[0];
        let img = &imgs[0];

        let movie_info = MovieInfo {
            name: name.to_string(),
            img: img.to_string(),
        };

        // println!("{:#?}", movie_info);
        assert_eq!(
            MovieInfo {
                name: "肖申克的救赎".to_string(),
                img: "https://img2.doubanio.com/view/photo/s_ratio_poster/public/p480747492.jpg"
                    .to_string()
            },
            movie_info
        );
        assert_eq!(vec![1], movie_ids);
        let info = "  张国荣 Leslie Cheung / 张丰毅 Fengyi Zha...<br>";
        let info = info.trim_start().trim_end_matches("...<br>");
        assert_eq!("张国荣 Leslie Cheung / 张丰毅 Fengyi Zha", info);

        println!("{:#?}", release_times);
        println!("{:#?}", movie_actors);
    }
}
