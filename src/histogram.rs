use plotters::prelude::*;

pub fn draw(
    mut language_stats: Vec<(String, usize)>,
    color: RGBColor,
    title: &str,
    group_threshold: usize,
) {
    let data = {
        language_stats.sort_by(|(_, a), (_, b)| b.cmp(a));

        let (big, small) = {
            let split_point = language_stats
                .iter()
                .position(|(_, stats)| *stats < language_stats[0].1.checked_div(group_threshold).unwrap_or(0))
                .unwrap_or(language_stats.len());
            language_stats.split_at_mut(split_point)
        };
        let mut big = big.to_vec();

        let others = small.iter().fold(0, |acc, (_, stats)| acc + stats);
        match big.iter().position(|(lang, _)| lang.eq(&"Others")) {
            Some(pos) => big[pos].1 += others,
            None => big.insert(0, ("Others".to_string(), others)),
        };
        big.sort_by(|(_, a), (_, b)| b.cmp(a));

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
