use std::sync::Arc;

use crossterm::style::{Color, StyledContent, Stylize};
use rayon::prelude::*;

use crate::{config::Orientation, util::AsciiArt};

pub trait Colorizer {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>>;
}
pub struct Default {}

impl Colorizer for Default {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let colors = &ascii_art.colors;
        ascii_art
            .text
            .par_iter()
            .map(|(idx, text)| -> StyledContent<String> {
                text.clone().with(*colors.get((*idx as usize) - 1).unwrap())
            })
            .collect::<Vec<StyledContent<String>>>()
    }
}

pub struct Flag {
    pub color_scheme: Arc<[Color]>,
    pub orientation: Orientation,
}

impl Flag {
    fn length_to_colors(&self, length: usize) -> Vec<Color> {
        let preset_len = self.color_scheme.len(); //6
        let center = preset_len / 2; // 4

        let repeats = length / preset_len; // 1
        let mut weights = [repeats].repeat(preset_len);
        let mut extras = length % preset_len; // 2
        if extras % 2 == 1 {
            extras -= 1;
            weights[center] += 1;
        }
        let mut border = 0;
        while extras > 0 {
            extras -= 2; //0
            weights[border] += 1; //
            weights[preset_len - border - 1] += 1;
            border += 1;
        }
        self.weights_to_colors(weights)
    }
    fn weights_to_colors(&self, weights: Vec<usize>) -> Vec<Color> {
        weights
            .into_par_iter()
            .enumerate()
            .flat_map(|(idx, weight)| {
                let mut v: Vec<Color> = [self.color_scheme[idx]].repeat(weight);
                v.fill(self.color_scheme[idx]);
                v
            })
            .collect::<Vec<Color>>()
    }
}

impl Colorizer for Flag {
    fn colorize(&self, ascii_art: &AsciiArt) -> Vec<StyledContent<String>> {
        let txt: String = ascii_art
            .text
            .clone()
            .into_par_iter()
            .map(|x| x.1)
            .collect();

        match self.orientation {
            Orientation::Horizontal => {
                let colors = self.length_to_colors(txt.par_lines().count());

                txt.par_lines()
                    .collect::<Vec<&str>>()
                    .par_iter()
                    .enumerate()
                    .map(move |(i, l)| ((*l).to_string() + "\n").with(colors[i]))
                    .collect::<Vec<_>>()
            }

            Orientation::Vertical => {
                //Requires txt has at least one line and is rectangular
                let colors = self.length_to_colors(ascii_art.width as usize);

                txt.par_lines()
                    .flat_map(|line| {
                        line.par_char_indices()
                            .map(|(idx, ch)| ch.to_string().with(colors[idx]))
                            .chain([String::from("\n").with(Color::Reset)])
                    })
                    .collect()
            }
        }
    }
}
