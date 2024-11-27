import matplotlib.pyplot as plt

# Example arrays
linear_totalcount = [300,620, 952, 1264, 1532, 1756]  # Third array (x-axis)
x = [1, 2, 3, 4, 5,6]  # First array (y-axis)
linear_success = [57, 76.6, 86.6, 92.5, 97.5,98.3]   # Second array (y-axis)
star_totalcount= [278,1499,1499,1499,1499,1499]
star_success = [24.2,100,100,100,100,100]

custom_totalcount = [301, 769,1072,1429,1741,1850]
custom_success= [55.8,80,86.6,95,100,100]

linear_totalcount_cache = [320,670, 1100, 1450, 1943, 2532]
linear_success_cache = [62, 85, 99, 100,100,100]

star_totalcount_cache = [300, 1950,1950,1950,1950,1950]
star_success_cache= [80,100,100,100,100,100]

custom_totalcount_cache= [340,900, 1400, 1900, 2233, 2567]
custom_success_cache = [65, 94,98,100,100,100]

# Plot the arrays
plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, linear_totalcount, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x, linear_totalcount_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (no.of messages)")
plt.title("comparison of total messages vs TTL in linear topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()



plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, star_totalcount, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x, star_totalcount_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (no.of messages)")
plt.title("comparison of total messages vs TTL in start topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()


plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, custom_totalcount, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x, custom_totalcount_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (no.of messages)")
plt.title("comparison of total messages vs TTL in custom topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()


plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, linear_success, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x, linear_success_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (% of query hits)")
plt.title("coparison of success vs TTL in linear topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()




plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, star_success, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x,star_success_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (% of query hits)")
plt.title("coparison of success vs TTL in star topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()



plt.figure(figsize=(8, 6))  # Set the figure size
plt.plot(x, custom_success, label="withcout caching", marker="o")  # Plot the first array
plt.plot(x, custom_success_cache, label="with caching", marker="s")  # Plot the second array

# Add labels, title, and legend
plt.xlabel("X-Axis (TTL)")
plt.ylabel("Y-Axis (% of query hits)")
plt.title("coparison of success vs TTL in custom topology")
plt.legend()

# Display the grid and show the plot
plt.grid(True)
plt.show()
