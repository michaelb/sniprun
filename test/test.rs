fn main() {
    let a = 4;
    let b = 5;
    println!("sum : {}", a + b);

    let a = 8 * 0 * 4 * 4;
    let b = 10;
    println!("div : {}", b / a);

    fn have_two() -> u16 {
        return 2;
    }
    let i = std::iter::once(have_two() * 2)
        .map(|u| u * u)
        .next()
        .unwrap();
    println!("hello n° {}", i + 0);
}
