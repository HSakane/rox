print now();

class SuperClass {
    fun super_method(a, b, c) {
        print "called \"super_method\" (" + (a + b + c) + ")";
    }
}

class SubClass < SuperClass {
    fun init(a, b) {
        this.a = [a, b, a, b, a];
        this.b = b;
        fun inner(a) {
            print "inner: " + a;
        }
        this.f = inner;
    }
    fun bacon(a, b, c) { print "bacon"; }
    fun eggs() { print "eggs"; }
    fun sub_method(a, b, c) {
        super.super_method(1, 2, 3);
        var x = a * b * c;
        var y = this.first * this.second;
        return "result: \"" + (x + y) + "\"";
    }
}

fun function() {
    print "called function";
}

var instance = SubClass("hoge", function);
instance.first = 10;
instance.second = 20;

instance.bacon(1, 2, 3);
instance.eggs();
print instance.sub_method(3,4,7);

print instance.a;
instance.b();
instance.f("fuga");
instance.super_method(11, 22, 33);

print now();
