class A inherits IO {
    a: Int <- 0;

    a_func(): Int {
        a
    };
};

class B inherits A {
    b: Int <- 1;

    b_func(): Int {
        a + a_func() + b
    };
};

class C inherits B {
    c: Int <- 2;

    c_func(): Int {
        a + b + c + a_func() + b_func()
    };

    print_c(): Object {
        out_int(c_func())
    };
};
