#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

void print_second(unsigned long num)
{
    printf("%lu\n", num);
    sleep(1);
}

int main(int argc, char *argv[])
{
    unsigned long num_seconds;
    if (argc != 2 || (num_seconds = strtoul(argv[1], NULL, 10)) == 0)
    {
        fprintf(stderr, "Usage: %s <seconds to sleep>\n", argv[0]);
        exit(1);
    }
    for (unsigned long i = 0; i < num_seconds; i++)
    {
        print_second(i);
    }
    return 0;
}