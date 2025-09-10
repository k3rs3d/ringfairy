# `ringfairy` 🧚

[![Build status](https://github.com/k3rs3d/ringfairy/actions/workflows/ci.yml/badge.svg)](https://github.com/k3rs3d/ringfairy/actions)
[![GitHub Release](https://img.shields.io/github/v/release/k3rs3d/ringfairy)](https://github.com/k3rs3d/ringfairy/releases)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/k3rs3d/ringfairy/latest)](https://github.com/k3rs3d/ringfairy/commits/main/)
[![GitHub last commit](https://img.shields.io/github/last-commit/k3rs3d/ringfairy)](https://github.com/k3rs3d/ringfairy/commits/main/)

[![Crates.io Total Downloads](https://img.shields.io/crates/d/ringfairy)](https://crates.io/crates/ringfairy)
[![Crates.io Size](https://img.shields.io/crates/size/ringfairy)](https://crates.io/crates/ringfairy)
[![License](https://img.shields.io/badge/license-GPL_v3.0-blue.svg)](https://github.com/k3rs3d/ringfairy/LICENSE)

This is a webring generator written in Rust. It's similar to a static site generator, but specialized for webrings. 

Unlike most webrings which rely on some kind of server-side code to redirect visitors, this uses HTML redirects. The static approach allows for simpler hosting requirements (it can be hosted on Neocities, GitHub Pages, etc) since it eliminates the need for server-side processing. 

However, updating the webring will require you to regenerate the whole thing. This is quick & simple, and shouldn't need to be done frequently. But it is an extra step which conventional server-side systems might not have, unless you automate it yourself. 

## 🔮 What is a Webring?

A webring is a collection of websites linked together in a loop. Each website contains links to the previous and next websites in the ring. If you navigate far enough, eventually you end up back where you started! 

<a href="https://upload.wikimedia.org/wikipedia/commons/9/97/Webringwork.png"><img src="https://upload.wikimedia.org/wikipedia/commons/9/97/Webringwork.png" width="512" alt="Webring example graphic"/></a>

Webrings were popular in the early days of the internet as a way for website owners to promote each other's content and encourage community engagement. 

This is a tool for anyone who has some kind of personal website or blog and wishes to connect with others. You can use it to grow your own online community from scratch!

## 🔬 Features 

- Fast & Lightweight
- Ensures each site contains the webring links
- Fully customizable via templates
- Generates a [OPML](https://opml.org/) file with all sites that have a RSS feed
- Choice of command-line interface or config file
- Remote config file support as well
- Shuffle website sequence optionally
- HTML minification
- Auto-link website owner contact info
- Catches errors and duplicate entries
- Detailed logging

## 🔎 Webrings

Webrings using `ringfairy` (as far as I know):

- [Ghostring](https://ghostring.neocities.org) - horror themed sites
- [Craftering](https://craftering.systemcrafters.net) - for the [System Crafters](https://systemcrafters.net) community
- [Roboring](https://stellophiliac.github.io/roboring/) - for robot-aligned beings
- [Spellcircle](https://spellcircle.neocities.org/) - for witches, wizards, and other magical beings
- [shring](https://shring.sh) - for unix-y personal sites
- [Catppuccin Webring](https://ctp-webr.ing/)
- [hajakehä](https://hajakeha.usvs.xyz/) - for Finnish speakers

If you decide to launch your own webring with this tool, let me know and I'll list it here! 

## 🪄 Usage

- Download a release binary OR clone and build from source. 
- Modify the `websites.json` (by default) file to include the details of the websites you want to include in the webring. Each website must be added to the list.
- Modify the `config.json` (by default) file according to your needs. 
- (Optional) Customize pages by modifying the templates, located in the `data/templates` folder (by default). You can also use remote files as templates. See the "Templates" section below. 
- (Optional) Add any additional files into the `data/assets` folder (by default). Everything in this folder will simply be copied over into the output directory. Here you can add extras like images, HTML/CSS, etc. 
- Run `ringfairy` to generate the webring by writing HTML files containing the redirects. Each site will link to the next/previous site in the `websites.json` file, forming your webring!
- Host the generated files on your preferred hosting platform. 

## ⚙️ Command-Line Arguments

Command-line arguments take precedence over any settings in the config file. 

- *`-h`, `--help`*: Print help
- *`-c`, `--config`*: Specify the location of the optional config file. It can be remote; for example an HTTP link to an appropriate JSON file on Pastebin, GitHub, etc. 
- *`-l`, `--list`*: Specify the JSON or TOML file containing the list of websites. Default: `./websites.json`
- *`-o`, `--output`*: Define the output folder, where the generated files will be saved. Default: `./webring`
- *`-a`, `--assets`*: Specify the assets folder. Any files in here will be copied to the output folder. This lets you include any extra files you want, such as images or extra web pages, etc. Default: `./data/assets`
- *`-t`, `--templates`*: Specify path to the template folder. Use `redirect.html` for redirect pages (i.e. the HTML which composes the webring). Any extra pages can be added here if you want them to be populated with generated content. Default: `./data/templates`
- *`-u`, `--url`*: The base URL for the webring. Something like 'https://example.com'. 
- *`-n`, `--name`*: The name of the webring. Something like 'Ghostring'.
- *`-d`, `--description`*: A short description/about the webring.
- *`-m`, `--maintainer`*: The owner/maintainer of the webring, could be a person or an organization.
- *`-w`, `--website`*: The website link of the website owner, not the base URL of the webring.
- *`--skip-minification`*: Outputs pages without optimizing or modifying them. Try this if you want your generated files to be hand-editable later, or if you experience any unexpected issues with the output.
- *`--skip-verification`*: Generates files without checking for potential problems...unwise!
- *`--dry-run`*: Runs the application without outputting any files
- *`-s`, `--shuffle`*: Randomly shuffles the order of websites during generation. This is totally internal and does not affect the input list of websites; you can shuffle the same webring repeatedly without losing the original sequence. 
- *`-v`, `--verbose`*: Output information to the console. `-vv` for very verbose mode to see even more info. 
- *`-V`, `--version`*: Print version

- *`-A`, `--audit`*: Audit mode. Scrapes each website in the list, checking to see if the next/previous links can be found. Otherwise, the site won't be added to the webring for that build. This means you don't have to immediately remove non-compliant websites; sites simply won't show up until the links can be found. If you use this without verbose mode (`-v`), you might not see the results of the audit. Don't use audit mode if you're building the webring offline, or if you want the fastest possible build speed. 
- *`-M`, `--audit_retries_max`*: In audit mode, maximum number of times to try reconnecting to a site. Default: `2`
- *`-D`, `--audit_retries_delay`*: In audit mode, milisecond delay before trying to reconnect to an unresponsive site. Default: `100`
- *`-U`, `--client_user_agent`*: In audit mode, user-agent string to be used by the web scraper. 
- *`-H`, `--client_header`*: In audit mode, header string to be used by the web scraper. 

- *`-J`, `--json-string`*: Provide website data in JSON format. 
- *`-T`, `--toml-string`*: Provide website data in TOML format. 

### Note: Logging

By default, the application only logs error messages. 

By passing `-v`/`--verbose` (on the command line) or setting `"verbose": true` (in the config JSON), you can tell the application to show warn level logs. To show info level logs, pass `-vv`; for debug, `-vvv`. 

To save these logs to a file, you can redirect standard output and standard error to a file when running your application. For example:

```
$ ./ringfairy > log.txt 2>&1
```

## 🎭 Templates

Templates are located in the `./data/templates` folder by default; this path can be specified with the command-line argument `--templates`, or with `path_templates` in the config file.

Templates contain tags which will be replaced with generated content. You can customize generated files by adding content before/after the tags. The repo includes basic template examples to get you started. 

In the templates folder, `redirect.html` is used to generate each of the `next.html`/`previous.html` pages, containing the redirects for each website. The tag `{{ url }}` is inserted by the generator in each page, and that powers the webring.

Besides `redirect.html`, the templates folder can contain any other templates you want.

For instance, it's a good idea for a webring to have a central hub page listing all of the sites. You can put this on `index.html`, or create a dedicated page such as `list.html`, `table.html`, etc. ~~Simply use the tag `{{ table_of_sites }}` in the template, and `ringfairy` will generate a formatted list of the sites in the webring.~~

### Template Tags

The following tags are currently usable in templates: 

- *`{{ sites }}`* provides access to information about the sites in the webring.  
- *`{{ number_of_sites }}`* shows the current size of the webring.
- *`{{ current_time }}`* displays the time of generating, showing when the page was last updated. 
- *`{{ opml }}`* inserts the relative path of the ring's OPML file.
- *`{{ base_url }}`* prints the webring's main URL, as set by the user. 
- *`{{ ring_name }}`* displays the webring's title. 
- *`{{ ring_description }}`* shows the webring's description according to settings. 
- *`{{ ring_owner }}`* shows the name of the webring's owner.
- *`{{ ring_owner_site }}`* prints the URL of the webring owner's site. 
- *`{{ featured_site_name }}`* prints the name of the "featured site", which is random. 
- *`{{ featured_site_description }}`* prints the description of the random featured site. 
- *`{{ featured_site_url }}`* prints the URL of the random featured site. 

Right now, `{{ url }}` is a unique tag that only works in `redirect.html` for the next/previous links.

----------------------------

<details>
  <summary>🍄</summary>
<pre><code>
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
</code></pre>
</details>

## ✨ Contributing

Contributions are welcome! If you have any suggestions for improvements or new features, feel free to open an issue or submit a pull request.
