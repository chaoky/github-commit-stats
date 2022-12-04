use plotters::prelude::*;
use crate::github::LanguageStats;

pub fn draw(
    language_stats: &[(String, LanguageStats)],
    picker: impl FnMut(&(String, LanguageStats)) -> (String, usize),
    color: RGBColor,
    title: &str,
) {
    let data = {
        let mut data: Vec<_> = language_stats.iter().map(picker).collect();

        data.sort_by(|(_, a), (_, b)| b.cmp(a));

        let (mut big, small) = {
            let split_point = data
                .iter()
                .position(|(_, stats)| (*stats as f64) < (data[0].1 as f64) * 0.01)
                .unwrap();
            data.split_at_mut(split_point)
        };

        let unknown_pos = big
            .iter()
            .position(|(lang, _)| lang.eq(&"unknown"))
            .unwrap();
        let others = small.iter().fold(0, |acc, (_, stats)| acc + stats);
        big[unknown_pos].0 = "Others".to_string();
        big[unknown_pos].1 += others;

        big.to_vec()
    };

    let file_name = format!("{}.png", title.replace(' ', "_"));
    let root = BitMapBackend::new(&file_name, (900, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 50.0))
        .x_label_area_size(50)
        .y_label_area_size(90)
        .build_cartesian_2d(
            0usize..(data[0].1 as f64 * 1.1).round() as usize,
            (0usize..data.len() - 1).into_segmented(),
        )
        .unwrap();

    chart
        .configure_mesh()
        .max_light_lines(1)
        .y_labels(100)
        .disable_y_mesh()
        .y_label_formatter(&|x: &SegmentValue<usize>| match x {
            SegmentValue::CenterOf(x) => data[*x].0.to_owned(),
            _ => unreachable!(),
        })
        .draw()
        .unwrap();

    chart
        .draw_series(
            Histogram::horizontal(&chart).style(color.filled()).data(
                data.iter()
                    .enumerate()
                    .map(|(index, &(_, stats))| (index, stats)),
            ),
        )
        .unwrap();

    root.present().expect("Unable to write result to file");
}
