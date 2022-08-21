# Snapchat Memories Downloader

This application takes a zip archive of downloaded data and downloads all avaliable images and videos that have been saved in the app (known as "memories").

I wrote this tool mainly to learn how to use Rust to build and structure an application properly, so any improvements and suggestions are greatly appreciated.

I do not believe that using it violates Snapchat's terms of service but use it at your own risk. I have found that leaving the "thread_sleep" argument at 100ms means you are not rate limited.

## Usage

- Request to download your data from Snapchat.
- Download the .zip file from Snapchat when you recieve an email prompting you to do so.
- Run the following command:

`snapchat-memories-downloader.exe <relative path to zip file> <relative path to output directory> <thread sleep (ms)>`

- Your memories should be downloaded to the output directory. This takes a little while especially if you have a lot of memories saved.
