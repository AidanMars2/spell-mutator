use std::fs;
use types::Spell;

fn main() {
    let raw_spells = fs::read_to_string("../../../spells_raw.txt")
        .expect("failed to load raw spells");

    let spells = parse_spells(&raw_spells);

    fs::write(
        "../../../spells.json",
        serde_json::to_string(&spells).expect("failed to serialize spells")
    ).expect("failed to write spells to file")
}

fn parse_spells(raw_spells: &str) -> Vec<Spell> {
    let mut spells: Vec<Spell> = vec![];

    for line in raw_spells.lines().into_iter() {
        let parts: Vec<String> = line.split("\t")
            .map(|part| part.to_string())
            .collect();

        if parts.len() < 8 {
            continue
        }

        spells.push(
            Spell {
                name: parts[0].to_string(),
                level: parts[1].chars().next().unwrap_or(' ')
                    .to_digit(10).unwrap_or(0) as u8,
                school: parts[2].clone(),
                ritual: !parts[3].is_empty(),
                cast_time: parts[4].clone(),
                components: parts[5].clone(),
                concentration: !parts[6].is_empty(),
                source: parts[7].clone(),
                mutations: vec![],
            }
        )
    }
    println!("parsed {} spells", spells.len());
    spells
}
