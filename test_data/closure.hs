print now();
var globalGet = null;
var globalSet = null;
var globalOuter = null;

fun main() {
	var a = "initial_0001";
	var b = "initial_0002";
	var c = "initial_0003";
	
	fun outer() {
		var d = "initial_0004";
		
		fun set() {
			b = "updated_0002";
			d = "updated_0004";
		}
		fun get() {
			print "d: " + d;
			print "c: " + c;
			print "b: " + b;
			print "a: " + a;
		}
		
		globalSet = set;
		globalGet = get;
	}

	globalOuter = outer;
}

main();
globalOuter();
globalGet();
globalSet();
globalGet();

print now();
