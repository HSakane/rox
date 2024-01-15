print now();
fun reduce(arr, initial, f) {
	fun iter(arr, result) {
		if(len(arr) == 0) {
			return result;
		} else {
			return iter(rest(arr), f(result, first(arr)));
		}
	}
	return iter(arr, initial);
}

fun sum(arr) {
    fun add(initial, el) { return initial + el; }
    return reduce(arr, 0, add);
}

print sum([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]);
print now();
