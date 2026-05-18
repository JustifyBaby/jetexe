#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

#define DEFINE_SCAN_LOOP(type, format)                      \
	type scan_loop_##type(String prompt, String error_msg)  \
	{                                                       \
		type value;                                         \
		int c;                                              \
		while (true)                                        \
		{                                                   \
			printf("%s", prompt);                           \
			if (scanf(format, &value) == 1)                 \
				return value;                               \
			printf("%s", error_msg);                        \
			while (((c = getchar()) != '\n') && (c != EOF)) \
				;                                           \
		}                                                   \
	}

DEFINE_SCAN_LOOP(int, "%d");

bool is_between_int(int min, int x, int max)
{
	return (min <= x) && (x <= max);
}

int main()
{
	int ref = scan_loop_int("Enter num: ", "Enter number!!!");
	printf("Hello, World! REF: p%d\n", ref);
	return 0;
}