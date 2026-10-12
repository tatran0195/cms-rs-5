//! Branch domain business logic.

use std::collections::HashMap;
use cms_db::{branch::BranchQueries, page::PageQueries, project::ProjectQueries, PgPool};
use cms_entity::{
    branch::{
        BranchResponse, BranchWithProjectResponse, CreateBranchRequest, ListBranchesQuery,
        ListBranchesResponse, SetDefaultBranchRequest, UpdateBranchRequest,
    },
    common::PaginatedResponse,
};
use cms_error::AppError;

/// Business service for Branch operations.
#[derive(Debug)]
pub struct BranchService;

impl BranchService {
    /// Create a new branch
    pub async fn create_branch(
        pool: &PgPool,
        user_id: &str,
        project_id: &str,
        request: CreateBranchRequest,
    ) -> Result<BranchWithProjectResponse, AppError> {
        let project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if let Some(ref from_branch_id) = request.from_branch_id {
            if !from_branch_id.trim().is_empty() {
                return Self::duplicate_branch(pool, user_id, from_branch_id.trim(), &request.name).await;
            }
        }

        let base_slug = request.name.trim().to_lowercase().replace(' ', "-");
        let base_slug = if base_slug.is_empty() {
            "branch".to_string()
        } else {
            base_slug
        };
        let mut slug = base_slug.clone();
        let mut counter = 1;

        let branch = loop {
            match BranchQueries::create_with_slug(
                pool,
                project_id,
                &request.name,
                &slug,
                request.description.as_deref(),
                false,
                !request.is_protected,
            )
            .await
            {
                Ok(b) => break b,
                Err(AppError::Conflict(_)) => {
                    slug = format!("{base_slug}-{counter}");
                    counter += 1;
                    if counter > 50 {
                        return Err(AppError::Conflict(
                            "Failed to allocate a unique branch slug under concurrency".to_string(),
                        ));
                    }
                }
                Err(e) => return Err(e),
            }
        };

        let branch_count = BranchQueries::count_by_project(pool, project_id, None).await?;
        if branch_count == 1 {
            BranchQueries::set_default(pool, &branch.id, &branch.project_id).await?;
        }

        Ok(BranchWithProjectResponse {
            branch: branch.into(),
            project: project.into(),
        })
    }

    /// Get a branch by ID
    pub async fn get_branch(
        pool: &PgPool,
        _user_id: &str,
        branch_id: &str,
    ) -> Result<BranchWithProjectResponse, AppError> {
        let branch = BranchQueries::get_by_id(pool, branch_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;

        let project = ProjectQueries::get_by_id(pool, &branch.project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        Ok(BranchWithProjectResponse {
            branch: branch.into(),
            project: project.into(),
        })
    }

    /// Get a branch by slug
    pub async fn get_branch_by_slug(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        slug: &str,
    ) -> Result<BranchWithProjectResponse, AppError> {
        let project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let branch = BranchQueries::get_by_slug(pool, project_id, slug)
            .await?
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;

        Ok(BranchWithProjectResponse {
            branch: branch.into(),
            project: project.into(),
        })
    }

    /// Update a branch
    pub async fn update_branch(
        pool: &PgPool,
        _user_id: &str,
        branch_id: &str,
        request: UpdateBranchRequest,
    ) -> Result<BranchResponse, AppError> {
        let branch = BranchQueries::get_by_id(pool, branch_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;

        if let Some(ref name) = request.name {
            let new_slug = name.to_lowercase().replace(' ', "-");
            if new_slug != branch.slug
                && !BranchQueries::is_slug_available(
                    pool,
                    &branch.project_id,
                    &new_slug,
                    Some(branch_id),
                )
                .await?
            {
                return Err(AppError::Conflict(
                    "Branch with this slug already exists".to_string(),
                ));
            }
        }

        if let Some(is_protected) = request.is_protected {
            if !is_protected && branch.is_default {
                return Err(AppError::AccessDenied(
                    "Cannot make the default branch unprotected".to_string(),
                ));
            }
        }

        let updated = BranchQueries::update(
            pool,
            branch_id,
            request.name.as_deref(),
            request.description.as_deref(),
            None,
            request.is_protected.map(|p| !p),
        )
        .await?;

        Ok(updated.into())
    }

    /// Delete a branch
    pub async fn delete_branch(
        pool: &PgPool,
        _user_id: &str,
        branch_id: &str,
    ) -> Result<bool, AppError> {
        let branch = BranchQueries::get_by_id(pool, branch_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;

        if branch.is_default {
            return Err(AppError::AccessDenied(
                "Cannot delete the default branch".to_string(),
            ));
        }

        if branch.is_protected {
            return Err(AppError::AccessDenied(
                "Cannot delete a protected branch".to_string(),
            ));
        }

        let page_count = PageQueries::count_by_branch(pool, branch_id).await?;
        if page_count > 0 {
            return Err(AppError::Conflict(
                "Cannot delete a branch with pages. Delete or move pages first.".to_string(),
            ));
        }

        BranchQueries::delete(pool, branch_id).await
    }

    /// List branches for a project
    pub async fn list_branches(
        pool: &PgPool,
        _user_id: &str,
        query: ListBranchesQuery,
        page: u64,
        page_size: u64,
    ) -> Result<ListBranchesResponse, AppError> {
        let _project = ProjectQueries::get_by_id(pool, &query.project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let limit = page_size.max(1);
        let offset = page.saturating_sub(1) * limit;

        let branches = BranchQueries::get_by_project(
            pool,
            &query.project_id,
            query.search.as_deref(),
            Some(i64::try_from(limit.clamp(1, 1000)).unwrap_or(50)),
            Some(i64::try_from(offset).unwrap_or(0)),
        )
        .await?;

        let total =
            BranchQueries::count_by_project(pool, &query.project_id, query.search.as_deref())
                .await?;

        Ok(PaginatedResponse::new(
            branches.into_iter().map(Into::into).collect(),
            total as u64,
            page,
            page_size,
        ))
    }

    /// Set the default branch for a project
    pub async fn set_default_branch(
        pool: &PgPool,
        _user_id: &str,
        request: SetDefaultBranchRequest,
    ) -> Result<BranchResponse, AppError> {
        let branch = BranchQueries::get_by_id(pool, &request.branch_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;

        let updated =
            BranchQueries::set_default(pool, &request.branch_id, &branch.project_id).await?;

        Ok(updated.into())
    }

    /// Duplicate a branch (create a copy with all pages)
    #[allow(clippy::too_many_lines)]
    pub async fn duplicate_branch(
        pool: &PgPool,
        _user_id: &str,
        source_branch_id: &str,
        name: &str,
    ) -> Result<BranchWithProjectResponse, AppError> {
        let source_branch = BranchQueries::get_by_id(pool, source_branch_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Source branch not found".to_string()))?;

        let project = ProjectQueries::get_by_id(pool, &source_branch.project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let base_slug = name.trim().to_lowercase().replace(' ', "-");
        let base_slug = if base_slug.is_empty() {
            format!("{}-copy", source_branch.slug)
        } else {
            base_slug
        };
        let mut slug = base_slug.clone();
        let mut counter = 1;

        let new_branch = loop {
            match BranchQueries::create_with_slug(
                pool,
                &source_branch.project_id,
                name,
                &slug,
                source_branch.description.as_deref(),
                false,
                false,
            )
            .await
            {
                Ok(b) => break b,
                Err(AppError::Conflict(_)) => {
                    slug = format!("{base_slug}-{counter}");
                    counter += 1;
                    if counter > 50 {
                        return Err(AppError::Conflict(
                            "Failed to allocate a unique duplicated branch slug under concurrency"
                                .to_string(),
                        ));
                    }
                }
                Err(e) => return Err(e),
            }
        };

        let all_pages = PageQueries::get_by_project(pool, &source_branch.project_id).await?;
        let branch_pages: Vec<cms_entity::page::Page> = all_pages
            .into_iter()
            .filter(|p| p.branch_id == source_branch_id)
            .collect();

        let mut id_mapping: HashMap<String, String> = HashMap::new();
        let mut remaining = branch_pages;

        while !remaining.is_empty() {
            let mut made_progress = false;
            let mut next_remaining = Vec::new();

            for page in remaining {
                let can_create = page
                    .parent_id
                    .as_ref()
                    .is_none_or(|parent_id| id_mapping.contains_key(parent_id));

                if can_create {
                    let mapped_parent = page
                        .parent_id
                        .as_ref()
                        .and_then(|pid| id_mapping.get(pid).cloned());

                    let new_page = PageQueries::create(
                        pool,
                        &source_branch.project_id,
                        &new_branch.id,
                        page.language_id.as_deref(),
                        mapped_parent.as_deref(),
                        Some(&page.kind),
                        &page.slug,
                        &page.title,
                        page.description.as_deref(),
                        Some(&page.content),
                        page.icon.as_deref(),
                        page.config.as_ref(),
                        page.translation_key.as_deref(),
                        page.position,
                        page.is_published,
                    )
                    .await?;

                    id_mapping.insert(page.id.clone(), new_page.id);
                    made_progress = true;
                } else {
                    next_remaining.push(page);
                }
            }

            if !made_progress {
                for page in next_remaining {
                    let new_page = PageQueries::create(
                        pool,
                        &source_branch.project_id,
                        &new_branch.id,
                        page.language_id.as_deref(),
                        None,
                        Some(&page.kind),
                        &page.slug,
                        &page.title,
                        page.description.as_deref(),
                        Some(&page.content),
                        page.icon.as_deref(),
                        page.config.as_ref(),
                        page.translation_key.as_deref(),
                        page.position,
                        page.is_published,
                    )
                    .await?;
                    id_mapping.insert(page.id.clone(), new_page.id);
                }
                break;
            }

            remaining = next_remaining;
        }

        Ok(BranchWithProjectResponse {
            branch: new_branch.into(),
            project: project.into(),
        })
    }
}
