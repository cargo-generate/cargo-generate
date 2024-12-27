// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded "><a href="usage/index.html"><strong aria-hidden="true">3.</strong> Usage</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="usage/git-over-ssh.html"><strong aria-hidden="true">3.1.</strong> Git over SSH</a></li><li class="chapter-item expanded "><a href="usage/gitconfig-instead-of.html"><strong aria-hidden="true">3.2.</strong> .gitconfig insteadOf</a></li></ol></li><li class="chapter-item expanded "><a href="favorites.html"><strong aria-hidden="true">4.</strong> Favorites</a></li><li class="chapter-item expanded "><a href="templates/index.html"><strong aria-hidden="true">5.</strong> Templates</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="templates/builtin_placeholders.html"><strong aria-hidden="true">5.1.</strong> Builtin Placeholders</a></li><li class="chapter-item expanded "><a href="templates/template_defined_placeholders.html"><strong aria-hidden="true">5.2.</strong> Template Defined Placeholders</a></li><li class="chapter-item expanded "><a href="templates/ignoring.html"><strong aria-hidden="true">5.3.</strong> Ignoring Files</a></li><li class="chapter-item expanded "><a href="templates/include_exclude.html"><strong aria-hidden="true">5.4.</strong> Include/Exclude</a></li><li class="chapter-item expanded "><a href="templates/require_version.html"><strong aria-hidden="true">5.5.</strong> Require Version</a></li><li class="chapter-item expanded "><a href="templates/conditional.html"><strong aria-hidden="true">5.6.</strong> Conditionals</a></li><li class="chapter-item expanded "><a href="templates/scripting.html"><strong aria-hidden="true">5.7.</strong> Hooks</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="templates/scripting.hook-types.html"><strong aria-hidden="true">5.7.1.</strong> Hook types</a></li><li class="chapter-item expanded "><a href="templates/scripting.rhai-extensions.html"><strong aria-hidden="true">5.7.2.</strong> Rhai extensions</a></li><li class="chapter-item expanded "><a href="templates/scripting.mini-example.html"><strong aria-hidden="true">5.7.3.</strong> Mini example</a></li></ol></li><li class="chapter-item expanded "><a href="templates/authoring.html"><strong aria-hidden="true">5.8.</strong> Authoring</a></li></ol></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">6.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="license.html"><strong aria-hidden="true">7.</strong> License</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
