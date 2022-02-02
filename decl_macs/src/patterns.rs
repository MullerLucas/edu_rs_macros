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



// ================================================================================================
// Push-down Accumulation
// ================================================================================================

#[allow(unused_macros)]
macro_rules! init_array {
    (@accum (0, $_e:expr) -> ($($body:tt)*)) => { init_array!(@as_expr [$($body)*]) };
    (@accum (1, $e:expr) ->  ($($body:tt)*)) => { init_array!(@accum (0, $e) -> ($($body)* $e,)) };
    (@accum (2, $e:expr) ->  ($($body:tt)*)) => { init_array!(@accum (1, $e) -> ($($body)* $e,)) };
    (@accum (3, $e:expr) ->  ($($body:tt)*)) => { init_array!(@accum (2, $e) -> ($($body)* $e,)) };
    (@accum (4, $e:expr) ->  ($($body:tt)*)) => { init_array!(@accum (3, $e) -> ($($body)* $e,)) };

    (@as_expr $e:expr) => { $e };
    [$e:expr; $n:tt] => {
        {
            let e = $e;
            init_array!(@accum ($n, e.clone()) -> ())
        }
    };
}

#[test]
fn accum_test() {
    let _strings: [String; 3] = init_array![String::from("hi"); 3];
}



// ================================================================================================
// Repetition Replacement
// ================================================================================================

#[allow(unused_macros)]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => { $sub };
}



// ================================================================================================
// TT Bundling
// ================================================================================================

#[allow(unused_macros)]
macro_rules! call_a_or_b_on_tail {
    ((a: $a:ident, b: $b:ident), call a: $($tail:tt)*) => {
        $a(stringify!($($tail)*))
    };

    ((a: $a:ident, b: $b:ident), call b: $($tail:tt)*) => {
        $b(stringify!($($tail)*))
    };

    ($ab:tt, $_skip:tt $($tail:tt)*) => {
        call_a_or_b_on_tail!($ab, $($tail)*)
    };
}

#[allow(dead_code)]
fn compute_len(s: &str) -> Option<usize> {
    Some(s.len())
}

#[allow(dead_code)]
fn show_tail(s: &str) -> Option<usize> {
    println!("tail: {:?}", s);
    None
}

#[test]
fn bundling_test() {
    assert_eq!(
        call_a_or_b_on_tail!(
            (a: compute_len, b: show_tail),
            the recursive part that skips over all these
            tokens doesn t much care whether we will call a
            or call b: only the terminal rules care.
        ),
        None
    );
    assert_eq!(
        call_a_or_b_on_tail!(
            (a: compute_len, b: show_tail),
            and now, to justify the existence of two paths
            we will also call a: its input should somehow
            be self-referential, so let s make it return
            some eighty-six!
        ),
        Some(92)
    );
}
