-- 006_indexes: 索引

CREATE INDEX IF NOT EXISTS idx_collections_project_id ON collections(project_id);
CREATE INDEX IF NOT EXISTS idx_collections_updated_at ON collections(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_folders_collection_id ON folders(collection_id);
CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id);

CREATE INDEX IF NOT EXISTS idx_requests_collection_id ON requests(collection_id);
CREATE INDEX IF NOT EXISTS idx_requests_folder_id ON requests(folder_id);
CREATE INDEX IF NOT EXISTS idx_requests_updated_at ON requests(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_executions_request_id ON request_executions(request_id);
CREATE INDEX IF NOT EXISTS idx_executions_created_at ON request_executions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_favorites_created_at ON favorites(created_at DESC);
