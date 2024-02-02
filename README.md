# Rustring: Static Webring Generator

This is a simple implementation of a webring generator using Rust. Unlike traditional webrings which rely on server-side processing (such as PHP) to handle redirects, this implementation generates static HTML files with redirects embedded directly into the files. 

This static approach allows for simpler hosting requirements (it can be hosted on GitHub Pages, etc), plus better performance as it eliminates the need for server-side processing. 

The downside to this approach is that updating the webring will require you to regenerate the whole thing. This is a simple process and shouldn't need to be done frequently, but it's still an extra step which conventional webring systems might not have. 

## What's a Webring?

A webring is a collection of websites linked together in a circular structure. Each website contains links to the previous and next websites in the ring. So, if you navigate far enough along the ring, eventually you end up back where you started! 

Webrings were popular in the early days of the internet as a way for website owners to promote each other's content and foster community engagement.

## How This Implementation Differs

Rustring differs from traditional webrings in a couple of ways:

### No Server-Side Processing: 

Instead of relying on server-side processing (such as a PHP script that handles redirects), this webring generates static HTML files with redirects embedded directly into the files. There is no need for server-side scripting to handle the redirects. Rustring operates entirely using static files. This simplifies hosting requirements and improves performance - the generated static HTML files can be hosted on barebones platforms (like GitHub Pages), allowing for easy and free hosting of the webring.

### Manual Updates: 

Rather than dynamically redirecting based on a list, the redirect pages that constitute the webring are generated all at once. This means that making changes to the webring might require you to regenerate all of the HTML (whereas webrings with server-side processing can be updated by simply modifying a file or database). So [re]generating the files adds an extra step compared to other webring approaches. 

Because processing is front-loaded to occur during the generation step, this is probably the most efficient way to build a webring in terms of performance (aside from doing the entire thing by hand). 

## Usage

To use this webring generator, follow these steps:

- Clone the repo.
- Modify the `websites.json` file to include the details of the websites you want to include in the webring.
- (Optional) Customize the generated HTML by modifying the templates, located in the `templates` folder. 
- Run the application to generate HTML files containing the redirects. Each site will link to the next/previous site in the `websites.json` file, forming a webring! 
- Host the generated HTML files on your preferred hosting platform. 

## Contributing

Contributions are welcome! If you have any suggestions for improvements or new features, feel free to open an issue or submit a pull request.