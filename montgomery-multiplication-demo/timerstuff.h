#pragma once

#include <stdio.h>
#include <time.h>
enum { NS_PER_SECOND = 1000000000 };
// https://stackoverflow.com/questions/53708076/what-is-the-proper-way-to-use-clock-gettime
static void sub_timespec(struct timespec t1, struct timespec t2, struct timespec *td)
{
    td->tv_nsec = t2.tv_nsec - t1.tv_nsec;
    td->tv_sec  = t2.tv_sec - t1.tv_sec;
    if (td->tv_sec > 0 && td->tv_nsec < 0)
    {
        td->tv_nsec += NS_PER_SECOND;
        td->tv_sec--;
    }
    else if (td->tv_sec < 0 && td->tv_nsec > 0)
    {
        td->tv_nsec -= NS_PER_SECOND;
        td->tv_sec++;
    }
}

static struct timespec _TIMER;
void timer_start() {
    clock_gettime(CLOCK_REALTIME, &_TIMER);
}
void timer_print(const char *name) {
    struct timespec end, delta;
    clock_gettime(CLOCK_REALTIME, &end);
    sub_timespec(_TIMER, end, &delta);
    printf("TIME FOR %s: %d.%.9ld\n", name, (int)delta.tv_sec, delta.tv_nsec);
}
