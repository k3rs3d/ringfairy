# Rustic Ring

This is a webring generator written in Rust. 

Unlike most webrings which rely on server-side code (e.g. PHP, JS) to redirect visitors, this implementation pre-generates static HTML files. It's similar to a static site generator, but specialized for webrings.  

The static approach allows for simpler hosting requirements (it can be hosted on GitHub Pages, etc), plus better performance as it eliminates the need for server-side processing. 

The catch: updating the webring will require you to regenerate the whole thing. This is quick & simple, and shouldn't need to be done frequently, but it's still an extra step which conventional server-side systems might not have. 

## What's a Webring?

A webring is a collection of websites linked together in a loop. Each website contains links to the previous and next websites in the ring, so if you navigate far enough, eventually you end up back where you started! 

Webrings were popular in the early days of the internet as a way for website owners to promote each other's content and encourage community engagement. 

This is a tool for anyone who has some kind of personal website or blog and wishes to connect with others. You can use it to grow your own online community from scratch. 

## Features 

- Highly optimized
- Fully customizable via templates
- Ability to shuffle webring
- Choice of command-line interface or config file
- Remote config file support
- HTML minification
- Catches duplicate entries
- Auto-link website owner contact info
- Detailed logging

## Usage

- Clone the repo.
- Modify the `websites.json` (by default) file to include the details of the websites you want to include in the webring. Each website must be added to the list. 
- (Optional) Customize pages by modifying the templates, located in the `data/templates` folder (by default). You can also use remote files as templates. See the "Templates" section below. 
- (Optional) Add any additional files into the `data/assets` folder (by default). Everything in this folder will simply be copied over into the output directory. Here you can add extras like images, HTML/CSS, etc. 
- Run `rustring` to generate the webring by writing HTML files containing the redirects. Each site will link to the next/previous site in the `websites.json` file, forming your webring!
- Host the generated files on your preferred hosting platform. 

## Command-Line Arguments

Command-line arguments take precedence over any settings in the config file. 

- *`-c`, `--config`*: Specify the location of the optional config file. It can be remote; for example an HTTP link to an appropriate JSON file on Pastebin, GitHub, etc. 
- *`-l`, `--list`*: Specify the file containing the list of websites. Default: `./websites.json`
- *`-o`, `--output`*: Define the output folder, where the generated files will be saved. Default: `./webring`
- *`-a`, `--assets`*: Specify the assets folder. Any files in here will be copied to the output folder. This lets you include any extra files you want, such as images or extra web pages, etc. Default: `./data/assets`
- *`-t`, `--templates`*: Specify path to the template folder. Use `template.html` for redirect pages (i.e. the HTML which composes the webring). Any extra pages can be added here if you want them to be populated with generated content. Default: `./data/templates`
- *`--skip-verification`*: Generates files without checking for potential problems...unwise!
- *`--dry-run`*: Runs the application without outputting any files
- *`-s`, `--shuffle`*: Randomly shuffles the order of websites during generation. This is totally internal and does not affect the input list of websites; you can shuffle the same webring repeatedly without losing the original sequence. 
- *`-v`, `--verbose`*: Output information to the console
- *`-V`, `--version`*: Print version
- *`-h`, `--help`*: Print help

### Note: Logging

By default, the application only logs error messages. By passing `-v`/`--verbose` (on the command line) or setting `"verbose": true` (in the config JSON), you can tell the application to show info level logs. 

To save these logs to a file, you can redirect standard output and standard error to a file when running your application. For example:

```
$ ./rustring > log.txt 2>&1
```

## Templates

Templates are located in the `./data/templates` folder by default; this path can be specified with the command-line argument `--templates`, or with `path_templates` in the config file.

Templates contain tags which will be replaced with generated content. You can customize generated files by adding content before/after the tags. The repo includes basic template examples to get you started. 

In the templates folder, `template.html` is used to generate each of the `next.html`/`previous.html` pages, containing the redirects for each website. The tag `{{ url }}` is inserted by the generator in each page, and that powers the webring. 

Besides `template.html`, the templates folder can contain any other templates you want. 

### Template Tags

The following tags are currently usable in templates: 

- *`{{ table_of_sites }}`* produces a formatted HTML table listing information for all sites in the webring. 
- *`{{ number_of_sites }}`* shows the current size of the webring.
- *`{{ current_time }}`* displays the time of generating, showing when the page was last updated. 

Right now, `{{ url }}`` is a special tag that only works in `template.html` for the next/previous links.  

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
