# Rustring: Static Webring Generator

This is a webring generator written in Rust. 

Unlike most webrings which rely on server-side code (e.g. PHP, JS) to redirect visitors, this implementation pre-generates static HTML files. 

The static approach allows for simpler hosting requirements (it can be hosted on GitHub Pages, etc), plus better performance as it eliminates the need for server-side processing. 

The catch: updating the webring will require you to regenerate the whole thing. This is quick & simple and shouldn't need to be done frequently, but it's still an extra step which conventional server-side webring systems might not have. 

## What's a Webring?

A webring is a collection of websites linked together in a loop. Each website contains links to the previous and next websites in the ring, so if you navigate far enough along the ring, eventually you end up back where you started! 

Webrings were popular in the early days of the internet as a way for website owners to promote each other's content and encourage community engagement. 

This is a tool for anyone who has some kind of personal website or blog and wishes to connect with others. You can use a webring to grow your own online community from scratch. 

## How This Implementation Differs

### No Server-Side Processing: 

Instead of relying on a server-side application to process redirects, this webring generates HTML with the redirects embedded directly in the files.

This simplifies hosting requirements and improves performance. The generated files can be hosted on barebones platforms (like GitHub Pages) or on minimal devices, allowing for easy & free hosting. You can run it darn near anywhere. 

### Manual Updates: 

Updating your webring requires you to regenerate all of the HTML, whereas other webrings (with server-side processing) can be updated by simply modifying a file or database. This step can be automated, but it's still an extra step to be aware of. 

Because processing is front-loaded to occur during page generation, this is probably the most performant & secure method to build a webring aside from doing the entire thing by hand. 

## Usage

- Clone the repo.
- Modify the `websites.json` (by default) file to include the details of the websites you want to include in the webring. Each website must be added to the list. 
- (Optional) Add any extra files (such as CSS or images) into the `data/assets` folder (by default). Everything in this folder will simply be copied over into the output directory. Here you can add things like logo images, additional HTML/CSS, etc. 
- (Optional) Customize pages by modifying the templates, located in the `data/templates` folder (by default). You can also use remote files as templates. See the "Templates" section below. 
- Run `rustring` to generate the webring by writing HTML files containing the redirects. Each site will link to the next/previous site in the `websites.json` file, forming your webring!
- Host the generated files on your preferred hosting platform. 

## Command-Line Arguments

Command-line arguments take precedence over any settings in the config file. 

- *`-c`, `--config`*: Specify the location of the optional config file. It can be remote; for example an HTTP link to an appropriate JSON file on Pastebin, GitHub, etc. 
- *`-l`, `--list`*: Specify the file containing the list of websites. Default: `./websites.json`
- *`-o`, `--output`*: Define the output folder, where the generated files will be saved. Default: `./webring`
- *`-a`, `--assets`*: Specify the assets folder. Any files in here will be copied to the output folder. This lets you include any extra files you want, such as images or extra web pages, etc. 
- *`-r`, `--path-template-redirect`*: Specify path to the template for redirect pages (i.e. the HTML which composes the webring). Can be a local or remote HTML file. Default: `./data/templates/redirect_template.html`
- *`-i`, `--path-template-index`*: Specify path to the template for the main index page (i.e. the formatted list of all websites in the webring). Can be a local or remote HTML file. Default: `./data/templates/list_template.html`
- *`--skip-verification`*: Generates files without checking for potential problems...unwise!
- *`--dry-run`*: Runs the application without outputting any files
- *`-v`, `--verbose`*: Output information to the console
- *`-h`, `--help`*: Print help
- *`-V`, `--version`*: Print version

### Note: Logging

By default, the application only logs error messages. By passing `-v`/`--verbose` (on the command line) or setting `"verbose": true` (in the config JSON file), you can tell the application to show logs. 

To save logs to a file, you can redirect standard output and standard error to a file when running your application. For example:

```
$ ./rustring > log.txt 2>&1
```

## Templates

Templates are located in the `./data/templates` folder by default. You can also load them remotely by passing a URL into `--path-template-redirect` and/or `--path-template-index` (on the command line), or `filepath_template_redirect` and/or `filepath_template_index` (within the config file). 

Templates contain tags which will be replaced with generated content. You can customize generated files by adding content before/after the tags. The repo includes basic template examples to get you started. 

- *`list_template.html`*: This will be used to generate the main index page which lists all the websites in the webring. The tag `<!-- TABLE_OF_WEBSITES -->` will be replaced with the list. It can be specified with the command-line argument `--path-template-index`, or with `filepath_template_index` in the config file. 
- *`redirect_template.html`*: This template is for each of the `next.html`/`previous.html` pages generated for each website. The tag `<!-- REDIRECT -->` is used for the HTML that powers the webring. It can be specified with the command-line argument `--path-template-redirect`, or with `filepath_template_redirect` in the config file.

----------------------------

```
                    __
                  .-'  |
                 /   <\|
                /     \'
                |_.- o-o
                / C  -._)\
               / ',        |
              |   `-,_,__,'
              (,,)====[_]=|
                '.   ____/
                 | -|-|_
                 |____)_)
```

## Contributing

Contributions are welcome! If you have any suggestions for improvements or new features, feel free to open an issue or submit a pull request.
