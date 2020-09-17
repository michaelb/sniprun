fn get_infinite_two() -> u16 {5}




fn main() {

    fn get_infinite_two() -> u16{3}
    fn have_two() -> u16 {
        return 2;
    }

    let j = get_infinite_two();
    let k = have_two();


    let i = std::iter::once(get_infinite_two())
        .map(|u| u * u)
        .next()
        .unwrap();
    println!("hello n° {}", i );


    let stuff = 0;
    let a = 7/stuff;
    assert!(a > 0);
}
