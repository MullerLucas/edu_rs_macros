// [The Littel Book of Rust Macros](https://veykril.github.io/tlborm/introduction.html)

// ================================================================================================
// macro export
// ================================================================================================

mod outer {
    pub mod a {
        #[allow(unused_macros)]
        macro_rules! export_test_1 {
            () => { println!("Export test A"); };
        }

        #[macro_export]
        macro_rules! export_test_2 {
            () => { println!("Export test B"); };
        }
    }

    mod b {
        // use super::a::export_test_1;    // e: unresolved import
        use crate::export_test_2;


        #[allow(dead_code)]
        fn macro_user() {
            // export_test_1!();           // e: cannot determine resolution
            export_test_2!();
        }
    }

}



// ================================================================================================
// litteral matches
// ================================================================================================

#[allow(unused_macros)]
macro_rules! useless {
    () => {
        println!("Absolutely pointless...");
    };
    (1) => {
        println!("1 goes in none comes out...");
    };
    (2) => {
        println!("2 is greater than 1");
    };
    (a) => {
        println!("A should be ok or?")
    };
    (repeat) => {
        println!("I'm repeating, here we GO...");

        useless!(a);
    };
    (4 fn ['what "is going on"] @_@) => {
        4
    };
}


#[test]
fn useless_test() {
    useless!();
    useless!(1);
    useless!(2);
    useless!(a);
    useless!(repeat);

    useless!(4 fn ['what "is going on"] @_@);
    // useless!(3 fn ['what "is going on"] @_@);     // unknown token input

    useless![];
    // {} and (); always expand to an item
    useless!{}
    useless!();
}



// ================================================================================================
// simple matches
// ================================================================================================

#[allow(unused_macros)]
macro_rules! simple {
    // Add the expression $e three times
    ($e:expr) => {
        $e + $e + $e
    };
    // Adds expression $ea to expression $eb
    ($ea:expr => $eb:expr) => {
        $ea + $eb
    };
    // Summs the one or more given expressions $e
    (SUM -> $( $e:expr), +) => {
        // Enclose the expansion in a block so that we can use multiple statements
        {
            let mut res = 0;
            $(
                res += $e;
            )+
            res
        }
    };
    // Memberwise add repeted statements => must have same number of arguments
    (ADD -> $( $ea:expr ),* ; $( $eb: expr ),* ) => {
        $(
            println!("{} + {} = {}", $ea, $eb, ($ea + $eb));
        )*
    };
}

#[test]
fn simple_test() {
    assert_eq!(simple!(3), 9);
    assert_eq!(simple!(3 => 4), 7);
    assert_eq!(simple!(SUM -> 1, 2, 3, 4), 10);
    simple!(ADD -> 1, 2, 3, 4; 10, 20, 30, 40);
}



// ================================================================================================
// fragment specifiers
// ================================================================================================

#[allow(unused_macros)]
macro_rules! fraspec {
    ( BLOCK ->     $( $b:block    )* ) => { println!("BLOCK")    };
    ( EXPR ->      $( $e:expr     )* ) => { println!("EXPR")     };
    ( IDENT ->     $( $i:ident    )* ) => { println!("IDENT")    };
    ( ITEM ->      $( $i:item     )* ) => { println!("ITEM")     };
    ( LIFETIME ->  $( $l:lifetime )* ) => { println!("LIFETIME") };
    ( LITERAL ->   $( $l:literal  )* ) => { println!("LITERAL")  };
    // matches the contents of an *attribute*: a simple path, one without generic argumetns followed by a delimiter token, or an = followed by an literal expression
    ( META ->      $( $m:meta     )* ) => { println!("META")     };
    // since 2021: Allows or-patterns to be parsed -> changes the follow list of the fragment, preventing it from being followed by a | token
    ( PAT ->       $( $p:pat      )* ) => { println!("PAT")      };
    // get pre 2021 pat fragment behaviour (without or-patterns) back
    ( PAT_PARAM -> $( $( $p:pat_param )|+ )* ) => {};
    // matches *TypePath* style paths (includes function style trait forms: Fn -> ())
    ( PATH ->      $( $p:path     )* ) => { println!("PATH")     };
    // solely matches a statement without its trialing semicolon, unless its an item statement that requires a trailing semicolon
    ( STMT ->      $( $s:stmt     )* ) => { println!("STMT"); /* $($s)* */ };
    // token tree: Matches nearly anything while still allowing you to inspect the contents
    ( TT ->        $( $t:tt       )* ) => { println!("TT")     };
    // matches any kind of type expression
    ( TY ->        $( $t:ty       )* ) => { println!("TY")     };
    // matches a possibly empty visibility qualifier -> cannot wrap it in a direct repetition while matching
    ( VIS ->       $( $v:vis,     )*  ) => { println!("VIS")   };
}

#[test]
fn fraspec_tests() {
    // blocks
    // ------
    fraspec!(BLOCK ->
        {}
        { let my_var; }
        { 2 }
    );

    // expressions
    // -----------
    fraspec!(EXPR ->
        "literal"
        funcall()
        future.await
        break 'foo bar
    );

    // idents
    // ------
    fraspec!(IDENT ->
        // _ <- not an ident, it is a pattern
        foo
        // async
        O_________O
    );

    // items
    // -----
    fraspec!(ITEM ->
        struct Foo;
        enum Bar { Baz }
        impl Foo {}
        pub use crate::foo;
    );

    // lifetimes
    // ---------
    fraspec!(LIFETIME ->
        'static
        'a
        '_
    );

    // literals
    // --------
    fraspec!(LITERAL ->
        -1
        "hello world"
        b'b'
        true
    );

    // metas
    // -----
    fraspec!(META ->
        ASimplePath
        super::main
        path = "home"
        foo(bar)
    );

    // pat
    // ---
    fraspec!(PAT ->
        "literal"
        _
        0..5
        ref mut PatternsAreNice
        // 0 | 1 | 2 | 3            // should work with 2021 edition
    );

    // pat_params
    // ----------
    fraspec!(PAT_PARAM ->
        "literal"
        _
        0..5
        ref mut PatternsAreNice
        0 | 1 | 2 | 3
    );

    // paths
    // -----
    fraspec!(PATH ->
         ASimplePath
         ::A::B::C::D
         G::<eneri>::C
         FnMut(u32) -> ()
    );

    // statements
    // ----------
    fraspec!(STMT ->
        struct Foo;
        fn foo() {}
        let zig = 3
        // 3
        // 3;
        // if true {} else {}
        {}
    );

    // type expressions
    // ----------------
    fraspec!(TY ->
        foo::bar
        bool
        [u8]
        impl IntoIterator<Item = u32>
    );

    // type expressions
    // ----------------
    fraspec!(VIS ->
        ,
        pub,
        pub(crate),
        pub(in super),
        pub(in som_path),
    );
}




// ================================================================================================
// hygiene
// ================================================================================================

#[allow(unused_macros)]
macro_rules! hygiene {
    //
    ($a:ident, $e:expr) => {
        {
            // let a = 42;  a would be in a different syntax context, and could not be used at the call-site
            let $a = 42;
            $e
        }
    };
    // $crate meta-variable refers to the current crate
    (CRATE) => {
        $crate::basics::foo();
    };
}

#[test]
fn hygiene_tests() {
    let four = hygiene!(a, a / 10);

    hygiene!(CRATE);
}

#[allow(dead_code)]
pub fn foo() { println!("FOO"); }
