import numpy as np
import matplotlib.pyplot as plt


MAX_HARDNESS = 0.99
STABILITY_RANGE = 16.0


def var_hard_clip(x: float, hardness: float) -> float:
    x = max(-STABILITY_RANGE, min(STABILITY_RANGE, x))
    clamped_hardness = min(hardness, MAX_HARDNESS)
    fade = (hardness - clamped_hardness) / (1.0 - MAX_HARDNESS)
    softness = 1.0 - clamped_hardness * 0.5 - 0.5

    analog = x / (1.0 + abs(x) ** (1.0 / softness)) ** softness
    digital = max(-1.0, min(1.0, x))
    return analog * (1.0 - fade) + digital * fade


xs = np.linspace(-16.0, 16.0, 1024)
hs = np.linspace(0.0, 1.0, 128)

plt.plot([var_hard_clip(x, 0.0) for x in xs])
plt.plot([var_hard_clip(x, 0.1) for x in xs])
plt.plot([var_hard_clip(x, 0.2) for x in xs])
plt.plot([var_hard_clip(x, 0.3) for x in xs])
plt.plot([var_hard_clip(x, 0.4) for x in xs])
plt.plot([var_hard_clip(x, 0.5) for x in xs])
plt.plot([var_hard_clip(x, 0.6) for x in xs])
plt.plot([var_hard_clip(x, 0.7) for x in xs])
plt.plot([var_hard_clip(x, 0.8) for x in xs])
plt.plot([var_hard_clip(x, 0.99) for x in xs])
plt.plot([var_hard_clip(x, 1.0) for x in xs])
plt.show()
