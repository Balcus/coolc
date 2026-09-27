(* Example from the manual *)
class Silly {
    copy(): SELF_TYPE {
        self
    };
};

class Sally inherits Silly { };

class Main {
    x: Sally <- (new Sally).copy();

    main(): Sally {
        x
    };

    main2(): B {
        (new B).foo().bar()
    };
};

(* Method chaining with SELF_TYPE *)
class A {
    foo(): SELF_TYPE {
        self
    };
};

class B inherits A {
    bar(): SELF_TYPE {
        self
    };
};

(* SELF_TYPE as attribute type *)
class C {
    x: SELF_TYPE;
};

class D inherits A {
    x: SELF_TYPE;
    
    foo2(): D {
        x
    };

    bar(): A {
        foo()
    };
};

