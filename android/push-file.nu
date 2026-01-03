#!/usr/bin/env nu

# Push a file to the app's private data directory
# Usage: nu push-file.nu <local-file-path> [destination-filename] [--overwrite]

def main [
    file: path,              # Local file to push
    dest_name?: string,      # Optional destination filename (defaults to source filename)
    --overwrite (-o)         # Force upload even if file exists with same size
] {
    let package = "ly.hall.jetlagmobile"
    let app_dir = $"/data/user/0/($package)/files"

    # Get the filename from the path if dest_name not provided
    let filename = if ($dest_name == null) {
        $file | path basename
    } else {
        $dest_name
    }

    # Temp location on device (accessible by adb)
    let temp_path = $"/data/local/tmp/($filename)"
    let final_path = $"($app_dir)/($filename)"

    # Get local file size
    let local_size = (ls $file | get size.0)

    # Check if file exists on device and get its size
    let remote_size = (adb shell run-as $package stat -c %s $final_path
        | complete
        | if $in.exit_code == 0 { $in.stdout | str trim | into int } else { null })

    # Check if we should skip upload
    if $remote_size != null and $local_size == $remote_size and not $overwrite {
        print $"⊘ Skipping ($filename) - same size: ($local_size) bytes"
        return
    }

    if $remote_size != null and $local_size == $remote_size {
        print $"Overwriting ($file) to ($final_path) - same size: ($local_size) bytes"
    } else if $remote_size != null {
        print $"Updating ($file) to ($final_path) - ($remote_size) -> ($local_size) bytes"
    } else {
        print $"Pushing ($file) to ($final_path) - ($local_size) bytes"
    }

    # Step 1: Push to temp location
    print $"  1. Pushing to temp: ($temp_path)"
    adb push $file /data/local/tmp/ | ignore

    # Step 2: Ensure app directory exists and copy file
    print $"  2. Copying to app directory..."
    adb shell run-as $package mkdir -p $app_dir
    adb shell run-as $package cp $temp_path $final_path

    # Step 3: Clean up temp file
    print $"  3. Cleaning up temp file..."
    adb shell rm $temp_path

    print $"✓ Done"
}
