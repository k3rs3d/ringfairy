# Rustring: Static Webring Generator

This is a simple implementation of a webring generator using Rust. Unlike traditional webrings which rely on server-side code (such as PHP) to handle redirects, this implementation generates static HTML files with predetermined redirects.

The static approach allows for simpler hosting requirements (it can be hosted on GitHub Pages, etc), plus better performance as it eliminates the need for server-side processing. 

The downside: updating the webring will require you to regenerate the whole thing. This is a simple process and shouldn't need to be done frequently, but it's still an extra step which conventional server-side webring systems might not have. 

## What's a Webring?

A webring is a collection of websites linked together in a circular structure. Each website contains links to the previous and next websites in the ring. So, if you navigate far enough along the ring, eventually you end up back where you started! 

Webrings were popular in the early days of the internet as a way for website owners to promote each other's content and facilitate community engagement.

## How This Implementation Differs

### No Server-Side Processing: 

Instead of relying on server-side processing (such as a PHP script that handles redirects), this webring generates static HTML files with redirects embedded directly into the files. There's no need to run anything to handle the redirects; `rustring` operates entirely using static files. 

This simplifies hosting requirements and improves performance - the generated files can be hosted on barebones platforms (like GitHub Pages) or on minimal devices, allowing for easy & free hosting of the webring.

### Manual Updates: 

Making changes to the webring might require you to regenerate all of the HTML (whereas webrings with server-side processing can be updated by simply modifying a file or database). So [re]generating the files adds an extra step compared to other webring approaches. 

Because processing is front-loaded to occur during the generation step, this is probably the most efficient way to build a webring in terms of performance (aside from doing the entire thing by hand). 

## Usage

- Clone the repo.
- Modify the `websites.json` file to include the details of the websites you want to include in the webring.
- (Optional) Customize the generated HTML by modifying the templates, located in the `templates` folder. See the section below. 
- Run `rustring` to generate HTML files containing the redirects. Each site will link to the next/previous site in the `websites.json` file, forming your webring! 
- Host the generated HTML files on your preferred hosting platform. 

## Command-Line Arguments

- *`-l`/`--list`*: Specify the file containing the list of websites. Default: `./websites.json`
- *`-v`/`--verbose`*: Output information to the console
- *`--skip-verification`*: Generates files without checking for potential problems
- *`--dry-run`*: Runs the application without outputting any files

## Templates

Templates are located in the `./templates/` folder. They contain tags which will be replaced with generated content. You can customize the generated files by adding content before and after the tags. The repo includes basic template examples. 

- *`list_template.html`*: This will be used to generate the main index page which lists all the websites in the webring. The tag `<!-- TABLE_OF_WEBSITES -->` will be replaced with the list. 
- *`redirect_template.html`*: This template is for the `next.html`/`previous.html` pages generated for each website. The tag `<!-- REDIRECT -->` is used for the HTML that powers the webring.

## To Do 
- Ability to load website/template files over network
- Time tag to show when the page was generated
- Generate `about.html` and/or `stats.html`?
- Support CSV import 

## Contributing

Contributions are welcome! If you have any suggestions for improvements or new features, feel free to open an issue or submit a pull request.
