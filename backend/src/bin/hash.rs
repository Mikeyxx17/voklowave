// 密码哈希工具
// 用法：cargo run --bin hash -- 你的密码
// 输出：$2b$10$...

fn main() {
    let password = std::env::args().nth(1).expect("用法：hash <密码>");
    let hash = bcrypt::hash(&password, 10).expect("bcrypt 哈希失败");
    println!("{hash}");
}
