use std::collections::HashMap;

use slug::slugify;
use tera::{Context, Tera};

use config::Config;
use errors::{Result, ResultExt};
use content::Page;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TaxonomyKind {
    Tags,
    Categories,
}

/// A tag or category
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TaxonomyItem {
    pub name: String,
    pub slug: String,
    pub pages: Vec<Page>,
}

impl TaxonomyItem {
    pub fn new(name: &str, pages: Vec<Page>) -> TaxonomyItem {
        TaxonomyItem {
            name: name.to_string(),
            slug: slugify(name),
            pages,
        }
    }
}

/// All the tags or categories
#[derive(Debug, Clone, PartialEq)]
pub struct Taxonomy {
    pub kind: TaxonomyKind,
    // this vec is sorted by the count of item
    pub items: Vec<TaxonomyItem>,
}

impl Taxonomy {
    // TODO: take a Vec<&'a Page> if it makes a difference in terms of perf for actual sites
    pub fn find_tags_and_categories(all_pages: Vec<Page>) -> (Taxonomy, Taxonomy) {
        let mut tags = HashMap::new();
        let mut categories = HashMap::new();

        // Find all the tags/categories first
        for page in all_pages {
            if let Some(ref category) = page.meta.category {
                categories
                    .entry(category.to_string())
                    .or_insert_with(|| vec![])
                    .push(page.clone());
            }

            if let Some(ref t) = page.meta.tags {
                for tag in t {
                    tags
                        .entry(tag.to_string())
                        .or_insert_with(|| vec![])
                        .push(page.clone());
                }
            }
        }

        // Then make TaxonomyItem out of them, after sorting it
        let tags_taxonomy = Taxonomy::new(TaxonomyKind::Tags, tags);
        let categories_taxonomy = Taxonomy::new(TaxonomyKind::Categories, categories);

        (tags_taxonomy, categories_taxonomy)
    }

    fn new(kind: TaxonomyKind, items: HashMap<String, Vec<Page>>) -> Taxonomy {
        let mut sorted_items = vec![];
        for (name, pages) in &items {
            sorted_items.push(
                TaxonomyItem::new(name, pages.clone())
            );
        }
        sorted_items.sort_by(|a, b| b.pages.len().cmp(&a.pages.len()));

        Taxonomy {
            kind,
            items: sorted_items,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get_single_item_name(&self) -> String {
        match self.kind {
            TaxonomyKind::Tags => "tag".to_string(),
            TaxonomyKind::Categories => "category".to_string(),
        }
    }

    pub fn get_list_name(&self) -> String {
        match self.kind {
            TaxonomyKind::Tags => "tags".to_string(),
            TaxonomyKind::Categories => "categories".to_string(),
        }
    }

    pub fn render_single_item(&self, item: &TaxonomyItem, tera: &Tera, config: &Config) -> Result<String> {
        let name = self.get_single_item_name();
        let mut context = Context::new();
        context.add("config", config);
        // TODO: how to sort categories and tag content?
        // Have a setting in config.toml or a _category.md and _tag.md
        // The latter is more in line with the rest of Gutenberg but order ordering
        // doesn't really work across sections.
        context.add(&name, item);
        context.add("current_url", &config.make_permalink(&format!("{}/{}", name, item.slug)));
        context.add("current_path", &format!("/{}/{}", name, item.slug));

        tera.render(&format!("{}.html", name), &context)
            .chain_err(|| format!("Failed to render {} page.", name))
    }

    pub fn render_list(&self, tera: &Tera, config: &Config) -> Result<String> {
        let name = self.get_list_name();
        let mut context = Context::new();
        context.add("config", config);
        context.add(&name, &self.items);
        context.add("current_url", &config.make_permalink(&name));
        context.add("current_path", &name);

        tera.render(&format!("{}.html", name), &context)
            .chain_err(|| format!("Failed to render {} page.", name))
    }
}