#include <math.h>
#include <stdio.h>

typedef struct MYLOG_interface {
    int x;
} MYLOG_interface;

typedef struct MYPRINTF_interface {
    char text[81];
    int value;
} MYPRINTF_interface;

int MYLOG(MYLOG_interface* param) {
    printf("Calling log with %d\n", param->x);
    int res =  (int) log10(param->x);
    printf("result :  %d\n", res);
    return res;
}

int MYPRINTF(MYPRINTF_interface* param) {
    return printf(param->text, param->value);
}





