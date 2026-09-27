class A {};

class B inherits A {};

class Main {
    main(): Object {
        (new A)@B.type_name()
    };
};
