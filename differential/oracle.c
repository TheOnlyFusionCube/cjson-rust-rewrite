#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "cJSON.h"

int main(void)
{
    size_t length = 0;
    size_t capacity = 4096;
    char *buffer = (char *)malloc(capacity);
    if (buffer == NULL)
    {
        return 1;
    }

    int byte = 0;
    while ((byte = fgetc(stdin)) != EOF)
    {
        if (length + 2 > capacity)
        {
            size_t next = capacity * 2;
            char *grown = (char *)realloc(buffer, next);
            if (grown == NULL)
            {
                free(buffer);
                return 1;
            }
            buffer = grown;
            capacity = next;
        }
        buffer[length++] = (char)byte;
    }
    buffer[length] = '\0';

    cJSON *item = cJSON_Parse(buffer);
    if (item == NULL)
    {
        free(buffer);
        return 2;
    }

    char *printed = cJSON_PrintUnformatted(item);
    cJSON_Delete(item);
    free(buffer);
    if (printed == NULL)
    {
        return 3;
    }
    fputs(printed, stdout);
    cJSON_free(printed);
    return 0;
}
