pub mod path_search {

    use crate::mrt_paths::path_data::PathData;

    pub fn find_paths(path_data: &PathData) {
        path_data.find_origins_with_overlapping_paths();
    }
}
