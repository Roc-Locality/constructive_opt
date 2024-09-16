import sys
import math
import pandas as pd
import matplotlib.pyplot as plt

def main(data_csv, output_plot):
    # Read data from CSV
    df = pd.read_csv(data_csv)

    # # Convert cache_size to log2 scale for x-axis
    # df['log_cache_size'] = df['cache_size'].apply(lambda x: math.log2(x))

    # Filter to include only powers of 2
    df = df[df['cache_size'].apply(lambda x: (x & (x - 1)) == 0 and x != 0)]

    # Convert cache_size to log2 scale for x-axis
    df['log_cache_size'] = df['cache_size'].apply(lambda x: math.log2(x))


    # Plotting
    plt.figure(figsize=(12, 8))
    plt.plot(df['log_cache_size'], df['miss_ratio'], color='red', label='OPT Miss Ratio Curve')

    # Configure x-axis to show actual cache sizes
    x_ticks = df['log_cache_size']
    x_labels = df['cache_size']
    plt.xticks(ticks=x_ticks, labels=x_labels.astype(int), rotation=45)

    plt.xlabel('Cache Size')
    plt.ylabel('Miss Ratio')
    plt.title('OPT Miss Ratio Curve')
    plt.legend()
    plt.grid(True)
    plt.ylim(-0.03, 1.03)
    plt.tight_layout()

    # Annotate critical points
    # critical_points = df[df['cache_size'].isin([2, 4, 8, 16, 32])]  # Example critical points
    for _, row in df.iterrows():
        plt.annotate(f"{row['miss_ratio']:.2f}",
                     (row['log_cache_size'], row['miss_ratio']),
                     textcoords="offset points",
                     xytext=(0,10),
                     ha='center')

    plt.tight_layout()

    # Save the plot
    plt.savefig(output_plot)
    print(f"Plot saved to {output_plot}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python plot_opt_miss_ratio.py <data_csv> <output_plot>")
        sys.exit(1)

    data_csv = sys.argv[1]
    output_plot = sys.argv[2]
    main(data_csv, output_plot)