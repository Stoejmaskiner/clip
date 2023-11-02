from scipy.signal import cheby2, freqz
import matplotlib.pyplot as plt
import numpy as np

CUTOFF = 0.6
STOPBAND_ATTEN = 80
mirrored_cutoff = 1.0 - CUTOFF

b, a = cheby2(8, STOPBAND_ATTEN, CUTOFF)
sos = cheby2(8, STOPBAND_ATTEN, CUTOFF, output="sos")

w, h = freqz(b, a)
plt.plot(w / np.pi, 20 * np.log10(abs(h)))
plt.grid(which="both", axis="both")
plt.axvline(mirrored_cutoff)
plt.axvline(CUTOFF)
plt.axhline(-STOPBAND_ATTEN / 2)
plt.show()

print(sos)
