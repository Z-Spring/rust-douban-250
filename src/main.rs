#![allow(unused)]

use futures::sink::Send;
use reqwest::{Client, Result};
use rust_douban::movie::{get_movie_infos, write_to_file, Movie};
use scraper::{Html, Selector};
use std::error::Error;
use std::future::Future;
use std::{result, thread};
use tokio::io::split;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let movies = handle_data().await;
    println!("{:?}", movies);
    let data = match write_to_file(movies).await {
        Ok(_) => format!("success write json to file!"),
        Err(err) => format!("write json to file error: {}", err),
    };
    let elapsed_time = now.elapsed();
    println!("{}\n{:?}", data, elapsed_time);
}

async fn handle_data() -> Vec<Movie> {
    let (tx, mut rx) = mpsc::channel(250);

    let pages: [u8; 10] = [0, 25, 50, 75, 100, 125, 150, 175, 200, 225];
    for page in pages {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut movies = get_movie_infos(page).await;
            tx.send(movies).await.expect("send error");
        });
    }
    drop(tx);

    let mut movies: Vec<Movie> = vec![];
    loop {
        tokio::select! {
            Some(mut movie) = rx.recv() => {movies.append(&mut movie);}
            else => break
        };
    }

    /* while let Some(mut movie) = rx.recv().await {
        movies.append(&mut movie);
    } */

    movies
}
