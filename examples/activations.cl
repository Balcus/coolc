Class Foo {
    g(): Int { 1 };
    f(): Int { g() };
    mainFoo(): Int {
        {
            g();
            f();
        }
    };
};

Class Bar {
    g(): Int { 1 };
    f(x: Int): Int {
        if x = 0 then g() else f(x - 1) fi
    };
    mainBar(): Int {
        {
            f(3);
        }
    };
};