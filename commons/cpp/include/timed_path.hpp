// Where something moving along a timed path is at a given moment: the trip interpolation the web
// observer does (web/src/people.ts) for hosts written in C++ such as the Unreal plugin. A path is
// a polyline with the cumulative minutes, after departure, at which each vertex is reached
// (TCE's trips: plan §3.1, ADR-0003). Header-only, C++17, no dependencies.

#pragma once

#include <algorithm>
#include <cmath>
#include <cstddef>

namespace engine_bridge {

/// A position in metres.
struct Point {
    double x = 0.0;
    double y = 0.0;
};

/// The position at simulation minute `t` (fractional) on a path of `count` vertices: `xy` holds
/// x, y pairs and `minutes` the cumulative minutes at which each vertex is reached after
/// `depart`. Before departure it is the first vertex; after arrival, the last.
inline Point position_on_path(const float* xy, const float* minutes, std::size_t count,
                              double depart, double t) {
    if (count == 0) return {};
    const double since = t - depart;
    if (since <= 0.0) return {xy[0], xy[1]};
    for (std::size_t k = 1; k < count; ++k) {
        const double m0 = minutes[k - 1];
        const double m1 = minutes[k];
        if (since <= m1) {
            const double f = m1 > m0 ? (since - m0) / (m1 - m0) : 1.0;
            const double x0 = xy[2 * k - 2];
            const double y0 = xy[2 * k - 1];
            return {x0 + (xy[2 * k] - x0) * f, y0 + (xy[2 * k + 1] - y0) * f};
        }
    }
    return {xy[2 * count - 2], xy[2 * count - 1]};
}

/// The simulation minute now, estimated from the latest clock: its minute plus the real time
/// since it arrived at its speed (simulated minutes per real minute), never more than a real
/// second's worth ahead, since the next snapshot corrects it. A clock that is `still` stands
/// still: one paused, or one in Accelerated mode (wire 1.23), whose frames come a day apart, each
/// at midnight; so does one at Max, an infinite speed.
inline double estimate_minute(double clock_minute, bool still, double speed, double received_ms,
                              double now_ms) {
    if (still || !std::isfinite(speed)) return clock_minute;
    const double per_second = speed / 60.0;
    const double ahead = (now_ms - received_ms) / 1000.0 * per_second;
    return clock_minute + std::min(std::max(0.0, ahead), std::max(1.0, per_second));
}

} // namespace engine_bridge
