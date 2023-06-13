from scipy import signal as dsp
import numpy as np
import matplotlib.pyplot as plt


def print_info(band_freqs, fs):
    normalized = np.array(band_freqs) / fs
    print(f"/// pass-band: {normalized[0]}..{normalized[1]}")
    print(f"/// stop-band: {normalized[2]}..{normalized[3]}")


def decibels(x: float) -> float:
    return 20 * np.log10(x)


decibels = np.vectorize(decibels)


def plot_fir(h: np.ndarray | list[float]):
    fft = np.fft.fft(h, 2048)
    H = np.abs(fft)
    H = np.fft.fftshift(H)
    w = np.linspace(-sample_rate / 2, sample_rate / 2, len(H))
    plt.plot(w[1024:], decibels(H[1024:]), "-")


def ideal(f: float) -> float:
    if f < band_freqs[1]:
        return 1.0
    if f > band_freqs[2]:
        return 0.0001
    t = (f - band_freqs[1]) / (band_freqs[2] - band_freqs[1])
    return 1.0 - t + 0.0001


ideal = np.vectorize(ideal)

print("/// Generated with scripts/generate_filters.py")
print()

num_taps = 127
band_freqs = [0, 17850, 23025, 46050]
band_gains = [1.0, 1.0, 0.0, 0.0]
weights = [0.01, 1000.0]
sample_rate = 92100

# t = np.linspace(0, sample_rate / 2, 2048)
# plt.plot(t, decibels(ideal(t)))
h = dsp.remez(
    num_taps, band_freqs, band_gains[::2], weights, maxiter=2000, fs=sample_rate
)
# plot_fir(h)

m = dsp.minimum_phase(h, method="hilbert", n_fft=5096)
# plot_fir(m)

# legends = ["ideal", "remez", "remez minimum"]
# plt.legend(legends)
# plt.show()

# plt.plot(m)
# plt.show()

print_info(band_freqs, sample_rate)
print(f"pub const LP_FIR_2X_TO_1X_MINIMUM_LEN: usize = {len(m)};")
print(f"pub const LP_FIR_2X_TO_1X_MINIMUM: [f32; LP_FIR_2X_TO_1X_MINIMUM_LEN] = [")
for coeff in m:
    print(f"    {coeff},")
print("];")
print()

num_taps = 63
band_freqs = [0, 0.25, 0.5, 1.0]
band_gains = [1.0, 1.0, 0.0, 0.0]
weights = [0.01, 1000.0]
sample_rate = 2

# t = np.linspace(0, sample_rate / 2, 2048)
# plt.plot(t, decibels(ideal(t)))

h = dsp.remez(
    num_taps, band_freqs, band_gains[::2], weights, maxiter=2000, fs=sample_rate
)
# plot_fir(h)

m = dsp.minimum_phase(h, method="hilbert", n_fft=5096)
# plot_fir(m)

# legends = ["ideal", "remez", "remez minimum"]
# plt.legend(legends)
# plt.show()

# plt.plot(m)
# plt.show()

print_info(band_freqs, sample_rate)
print(f"pub const LP_FIR_4X_TO_2X_MINIMUM_LEN: usize = {len(m)};")
print(f"pub const LP_FIR_4X_TO_2X_MINIMUM: [f32; LP_FIR_4X_TO_2X_MINIMUM_LEN] = [")
for coeff in m:
    print(f"    {coeff},")
print("];")
print()
