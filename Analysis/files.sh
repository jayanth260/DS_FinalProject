#!/bin/bash

# Define the list of filenames
files=("file1.txt" "file2.txt" "file3.txt" "file4.txt" "file5.txt" "file6.txt" "file7.txt" "file8.txt" "file9.txt" "file10.txt" "file11.txt" "file12.txt" "file13.txt" "file14.txt" "file15.txt")

# Create a file to store the locations
output_file=$1

# Clear the output file if it exists
> "$output_file"

# Function to create random file at random location
create_random_file() {
    # Generate a random directory path under /tmp (you can change this to any base directory)
    random_dir="/tmp/$(openssl rand -hex 4)"

    # Make the directory
    mkdir -p "$random_dir"

    # Generate a random index for file name
    random_index=$((RANDOM % 15))

    # Get the file name from the list
    file_name="${files[$random_index]}"

    # Create the file with some random content
#    echo "Random content for $file_name" > "$random_dir/$file_name"
	random_size=$((RANDOM%10000 + 1))
head -c  $random_size /dev/urandom | base64 > "$random_dir/$file_name"
    # Write the file path toi the output file
    echo "$random_dir/$file_name" >> "$output_file"
}

random_number=$((RANDOM % 13 + 1))
# Generate 8 random subsets and create the files
for i in $(seq 1 $random_number); do
	echo "hi"
    create_random_file
done

# Print the file locations
echo "File locations have been written to $output_file."

