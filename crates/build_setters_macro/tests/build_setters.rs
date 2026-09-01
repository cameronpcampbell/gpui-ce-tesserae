use build_setters_macro::BuildSetters;

#[derive(BuildSetters)]
struct Foo {
    apple: String,
    pear: f32,
    #[nosetter]
    secret: bool,
}

#[test]
fn generates_setters_except_when_disabled() {
    let foo = Foo {
        apple: String::new(),
        pear: 0.0,
        secret: true,
    }
    .apple("Granny Smith")
    .pear(1.5_f32);

    assert_eq!(foo.apple, "Granny Smith");
    assert_eq!(foo.pear, 1.5);
    assert!(foo.secret);
}
