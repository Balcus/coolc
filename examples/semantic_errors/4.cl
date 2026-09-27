(* Wrong override *)

class A {
    foo(x: Int): Object {
        x
    };
};

class B inherits A {
    foo(x: Bool): Object {
        x
    };
};
