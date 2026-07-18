class Main inherits io {
    main(): Object {
        out_string("Hello World!\n");
    };
};

class Foo inherits IO {
    foo(): Object {
        out_string("Foo\n");
    };
    bar(x: bool): bool {
        x
    };
};