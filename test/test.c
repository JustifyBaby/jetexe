#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

int scan_loop_int(String prompt, String error_msg)
{
	int value;
    int c;
	while (true)
	{
		printf("%s", prompt);
		if (scanf("%d", &value) == 1)
			return value;
            
        printf("%s", error_msg);
		while (
            ( (c = getchar()) != '\n' ) &&
            ( c != EOF )
        )
			;
	}
}

bool is_between_int (int min, int x, int max)
{
	return (min <= x) && (x <= max);
}

int main()
{
	return 0;
}
