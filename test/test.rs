// will try to finish that later
fn get_infinite_two() {





fn main() {
   let stuff =

    fn have_two() -> u16 {
        return 2;
    }



    let i = std::iter::once(have_two())
        .map(|u| u * u)
        .next()
        .unwrap();
    println!("hello n° {}", i );


    let stuff = 0;
    let a = 7/stuff;
    assert!(a > 0);
