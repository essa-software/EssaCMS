# EssaCMS usage

## Basics

Directory structure:

-   `pages` - stores pages
-   `templates` stores, well, templates:
    -   `main.html` - used for everything
-   `public` - stores files like CSS, images, fonts,...

In template, there is a `{{content}}` marker which will be replaced with corresponding content from page.

For example:

```html
<!-- templates/main.html -->
<html>
    <head>
        ...
    </head>
    <body>
        <header>index | about | something | blablabla</header>
        {{content}}
    </body>
</html>
```

Then in pages you just do:

```html
<!-- pages/index.html -->
<h1>Index</h1>
```

```html
<!-- pages/about.html -->
<h1>About</h1>
<p>EssaCMS is nice even if doesn't do anything yet lalalal</p>
```

## TODO

-   more templates
-   macros, variables
-   menus
-   do we want posts
-   backend features (forms etc)
