use unindent::unindent;

fn main() {
    let string_literal = "\n    count = 0 \n    for i in range(1,40000000):\n        count += 1\n    print(count, end=\"\")";
    println!("before:\n{}\n\n", string_literal);
    println!("after: \n{}", unindent(string_literal));
}
