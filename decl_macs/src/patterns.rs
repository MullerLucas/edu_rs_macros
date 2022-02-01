// [The Little Book of Rust Macros](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html)


// ================================================================================================
// Callbacks
// ================================================================================================

#[allow(unused_macros)]
macro_rules! call_with_larch {
    ($callback:ident) => { $callback!(larch) };
}

#[allow(unused_macros)]
macro_rules! expand_to_larch {
    () => { larch };
}

#[allow(unused_macros)]
macro_rules! recognize_tree {
    (larch) => { println!("#1, the Larch.") };
    (redwood) => { println!("#2, the Mighty Redwood.") };
    (fir) => { println!("#3, the Fir.") };
    (chestnut) => { println!("#4, the Horse Chestnut.") };
    (pine) => { println!("#5, the Scots Pine.") };
    ($($other:tt)*) => { println!("I don't know;  smoe knid of birch maybe?") };
}

// Using tt repetition, to forward arbitrary arguments to a callback
#[allow(unused_macros)]
macro_rules! callback {
    ( $callback:ident( $($args:tt)* )) => {
        $callback!( $($args)* )
    };
}

#[test]
fn test_callbacks () {
    // Impossible to pass information to a macro from the expansion of another macro, due to the order that macros are expanded in
    recognize_tree!(expand_to_larch!());

    call_with_larch!(recognize_tree);

    callback!(callback(println("Zes, this *was* unnecessary.")))
}



// ================================================================================================
// Incremental TT Munchers
// ================================================================================================

#[allow(unused_macros)]
macro_rules! mixed_rules {
    () => {};
    (trace $name:ident; $($tail:tt)*) => {
        {
            println!(concat!(stringify!($name), " = {:?}"), $name);
            mixed_rules!($($tail)*);
        }
    };

    (trace $name:ident = $init:expr; $($tail:tt)*) => {
        let $name = $init;
        println!(concat!(stringify!($name), " = {:?}"), $name);
        mixed_rules!($($tail)*);
    };
}

#[test]
fn text_tt_muncher() {
    mixed_rules!(trace a = 5; trace b = 8;);
}



// ================================================================================================
// Internal Rules
// ================================================================================================

// @ = standard way to declare an internal rule
#[allow(unused_macros)]
macro_rules! foo {
    (@as_expr $e:expr) => { $e };

    ($($tts:tt)*) => {
        foo!(@as_expr $($tts)*)
    };
}
