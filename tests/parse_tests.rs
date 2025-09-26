use ru_ip_dump::{parse_row, AsnRow};
use std::net::Ipv4Addr;

#[test]
fn parses_ru_rows_and_filters_others() {
    let us = "1.1.1.0\t1.1.1.255\t1337\tUS\tCloud";
    assert!(parse_row(us).is_none());

    let ru = "5.255.192.0\t5.255.255.255\t13238\tRU\tYandex";
    let got = parse_row(ru).expect("should parse RU row");

    let expected = AsnRow {
        first_ip: Ipv4Addr::new(5, 255, 192, 0),
        last_ip: Ipv4Addr::new(5, 255, 255, 255),
        number: 13238,
        country: "RU".into(),
        description: "Yandex".into(),
    };

    assert_eq!(got, expected);
}
