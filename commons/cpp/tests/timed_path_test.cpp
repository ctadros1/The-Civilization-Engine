// Tests of timed_path.hpp with the cases the web observer's tests use (web/tests/people.test.ts),
// so the C++ and TypeScript interpolation agree. Run by commons-cpp's cargo test.

#include <cmath>
#include <cstdio>

#include "timed_path.hpp"

namespace eb = engine_bridge;

static int failures = 0;

#define CHECK(condition)                                                                         \
    do {                                                                                         \
        if (!(condition)) {                                                                      \
            std::printf("FAILED %s:%d: %s\n", __FILE__, __LINE__, #condition);                   \
            ++failures;                                                                          \
        }                                                                                        \
    } while (0)

static bool near(double a, double b) { return std::fabs(a - b) < 1e-9; }

static bool at(eb::Point p, double x, double y) { return near(p.x, x) && near(p.y, y); }

int main() {
    // A trip departing at minute 100: (0,0), then (10,0) a minute later, then (10,20) at 3.
    const float xy[] = {0, 0, 10, 0, 10, 20};
    const float minutes[] = {0, 1, 3};
    CHECK(at(eb::position_on_path(xy, minutes, 3, 100, 90), 0, 0));
    CHECK(at(eb::position_on_path(xy, minutes, 3, 100, 100.5), 5, 0));
    CHECK(at(eb::position_on_path(xy, minutes, 3, 100, 102), 10, 10));
    CHECK(at(eb::position_on_path(xy, minutes, 3, 100, 500), 10, 20));
    CHECK(at(eb::position_on_path(xy, minutes, 0, 100, 101), 0, 0));
    // Two vertices reached at the same minute: the later one, not a division by zero.
    const float same[] = {0, 0, 0};
    CHECK(at(eb::position_on_path(xy, same, 3, 100, 100.5), 10, 20));

    // The clock runs on at its speed (here 16 simulated minutes a real second), stops while
    // paused, and never runs more than a real second ahead.
    CHECK(near(eb::estimate_minute(1000, false, 960, 0, 500), 1008));
    CHECK(near(eb::estimate_minute(1000, true, 960, 0, 500), 1000));
    CHECK(near(eb::estimate_minute(1000, false, 960, 0, 60000), 1016));
    CHECK(near(eb::estimate_minute(1000, false, 96, 0, 60000), 1001.6));

    if (failures == 0) std::printf("timed_path_test: ok\n");
    return failures == 0 ? 0 : 1;
}
