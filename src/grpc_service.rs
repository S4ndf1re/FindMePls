use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::BusinessRules;

pub use self::find_me_pls::find_me_pls_server::FindMePlsServer;
use self::find_me_pls::{
    find_me_pls_server::FindMePls, AddItemToCollectionRequest, Categories, Category, Collection,
    Collections, DeleteItemRequest, Empty, GetCollectionRequest, GetItemRequest, Item, Items,
    QueryItemsRequest, RemoveItemFromCollectionRequest,
};

pub mod find_me_pls {
    #![allow(non_snake_case)]
    tonic::include_proto!("find_me_pls");
}

pub struct FindMePlsService {
    business_rules: Option<Arc<BusinessRules>>,
}

impl FindMePlsService {
    pub fn new(business_rules: Arc<BusinessRules>) -> Self {
        Self {
            business_rules: Some(business_rules),
        }
    }
}

impl Default for FindMePlsService {
    fn default() -> Self {
        Self {
            business_rules: None,
        }
    }
}

#[tonic::async_trait]
impl FindMePls for FindMePlsService {
    async fn new_item(&self, request: Request<Item>) -> Result<Response<Item>, Status> {
        let result = self
            .business_rules
            .as_ref()
            .map(|t| t.add_item(request.into_inner().into()));
        match result {
            Some(item_res) => {
                let result = item_res.await;
                match result {
                    Ok(item) => Ok(Response::new(item.into())),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn get_all_items(&self, _request: Request<Empty>) -> Result<Response<Items>, Status> {
        let items_res = self.business_rules.as_ref().map(|t| t.get_all_items());
        match items_res {
            Some(items_res) => {
                let result = items_res.await;
                match result {
                    Ok(items) => Ok(Response::new(Items {
                        items: items.into_iter().map(Into::into).collect(),
                    })),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn get_item(&self, request: Request<GetItemRequest>) -> Result<Response<Item>, Status> {
        let item_res = self
            .business_rules
            .as_ref()
            .map(|t| t.get_item(request.into_inner().id));
        match item_res {
            Some(item_res) => {
                let result = item_res.await;
                match result {
                    Ok(item) => Ok(Response::new(item.into())),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn query_items(
        &self,
        request: Request<QueryItemsRequest>,
    ) -> Result<Response<Items>, Status> {
        let query = request.into_inner().query;
        let items_res = self.business_rules.as_ref().map(|t| t.find_items(query));
        match items_res {
            Some(items_res) => {
                let result = items_res.await;
                match result {
                    Ok(items) => Ok(Response::new(Items {
                        items: items.into_iter().map(Into::into).collect(),
                    })),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn delete_item(
        &self,
        request: Request<DeleteItemRequest>,
    ) -> Result<Response<Item>, Status> {
        let item_res = self
            .business_rules
            .as_ref()
            .map(|t| t.delete_item(request.into_inner().id));
        match item_res {
            Some(item_res) => {
                let result = item_res.await;
                match result {
                    Ok(item) => Ok(Response::new(item.into())),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn new_category(&self, request: Request<Category>) -> Result<Response<Category>, Status> {
        let result = self
            .business_rules
            .as_ref()
            .map(|t| t.new_category(request.into_inner().into()));

        match result {
            Some(category_res) => {
                let result = category_res.await;
                match result {
                    Ok(category) => Ok(Response::new(category.into())),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn get_all_categories(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Categories>, Status> {
        let result = self.business_rules.as_ref().map(|t| t.get_all_categories());

        match result {
            Some(categories_res) => {
                let result = categories_res.await;
                match result {
                    Ok(categories) => Ok(Response::new(Categories {
                        categories: categories.into_iter().map(Into::into).collect(),
                    })),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn new_collection(
        &self,
        request: Request<Collection>,
    ) -> Result<Response<Collection>, Status> {
        let result = self
            .business_rules
            .as_ref()
            .map(|busi| busi.new_collection(request.into_inner().into()));

        match result {
            Some(coll) => {
                let collection = coll.await;
                match collection {
                    Ok(collection) => Ok(Response::new(collection.into())),
                    Err(e) => Err(Status::from_error(e.into())),
                }
            }
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn get_all_collections(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Collections>, Status> {
        let result = self
            .business_rules
            .as_ref()
            .map(|busi| busi.get_all_collections());
        match result {
            Some(future) => future
                .await
                .map(|c| {
                    Response::new(Collections {
                        collections: c.into_iter().map(Into::into).collect(),
                    })
                })
                .map_err(|e| Status::from_error(e.into())),
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn get_collection(
        &self,
        request: Request<GetCollectionRequest>,
    ) -> Result<Response<Collection>, Status> {
        let id = request.into_inner().id;

        let future = self.business_rules.as_ref().map(|b| b.get_collection(id));
        match future {
            Some(future) => future
                .await
                .map(|c| Response::new(c.into()))
                .map_err(|e| Status::from_error(e.into())),
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn add_item_to_collection(
        &self,
        request: Request<AddItemToCollectionRequest>,
    ) -> Result<Response<Empty>, Status> {
        let add_item_request = request.into_inner();
        let item_id = add_item_request.item_id;
        let collection_id = add_item_request.collection_id;

        let future = self
            .business_rules
            .as_ref()
            .map(|b| b.add_item_to_collection(item_id, collection_id));

        match future {
            Some(future) => future
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|e| Status::from_error(e.into())),
            None => Err(Status::internal("Business rules not initialized")),
        }
    }

    async fn remove_item_from_collection(
        &self,
        request: Request<RemoveItemFromCollectionRequest>,
    ) -> Result<Response<Empty>, Status> {
        let remove_item_request = request.into_inner();
        let item_id = remove_item_request.item_id;
        let collection_id = remove_item_request.collection_id;

        let future = self
            .business_rules
            .as_ref()
            .map(|b| b.remove_item_from_collection(item_id, collection_id));

        match future {
            Some(future) => future
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|e| Status::from_error(e.into())),
            None => Err(Status::internal("Business rules not initialized")),
        }
    }
}
