// used in unit tests in gdf_responses and gdf_agent
#[macro_export]
macro_rules! translation_tests_assertions {
    ($response_type:ty, $str_before_translation:expr, $str_after_translation_expected:expr) => {
        let mut struct_to_translate: $response_type =
            serde_json::from_str($str_before_translation)?;
        let struct_after_translation_expected: $response_type =
            serde_json::from_str($str_after_translation_expected)?;
        let mut translations_map = struct_to_translate.to_translation();

        println!("{:#?}", struct_to_translate);

        dummy_translate(&mut translations_map);
        struct_to_translate.from_translation(&translations_map);
        let str_after_translation_real = serde_json::to_string(&struct_to_translate)?;

        assert_eq!(
            normalize_json(&$str_after_translation_expected),
            normalize_json(&str_after_translation_real)
        );

        assert_eq!(struct_to_translate, struct_after_translation_expected);

        println!("{:#?}", struct_to_translate);
    };

    ($response_type:ty, $str_before_translation:expr, $str_after_translation_expected:expr, "no_string_comparison") => {
        let mut struct_to_translate: $response_type =
            serde_json::from_str($str_before_translation)?;
        let struct_after_translation_expected: $response_type =
            serde_json::from_str($str_after_translation_expected)?;
        let mut translations_map = struct_to_translate.to_translation();

        println!("{:#?}", struct_to_translate);

        dummy_translate(&mut translations_map);
        struct_to_translate.from_translation(&translations_map);

        assert_eq!(struct_to_translate, struct_after_translation_expected);

        println!("{:#?}", struct_to_translate);
    };
}

// used in gdf_responses
// definying this function with generics is quite tricky becase of calling <<DeserializedStructType>>::new
// macro is good way here how to prevent writing same function multiple times
// original function was something like this, see usage of DeserializeOwned
/* fn check_gdf_zip_glob_files<T>(glob_exp: &str, contains_array: bool) -> Result<()>
where
    T: serde::de::DeserializeOwned + Serialize, // see https://serde.rs/lifetimes.html !
{
    for entry in glob(glob_exp)? {
        let path = entry?;

        let file_name = path.as_path().to_str().unwrap();

        if contains_array == false
            && (file_name.contains("_entries_") || file_name.contains("_usersays_"))
        {
            continue; // if not processing arrays (entity entries or intent utterances) skip respective files!
        }

        debug!("processing file {}", file_name);
        let file_str = fs::read_to_string(file_name)?;

        let deserialized_struct: T = serde_json::from_str(&file_str)?;

        let serialized_str = serde_json::to_string(&deserialized_struct).unwrap();
        let comparison_result = assert_json_eq_no_panic(
            &serde_json::from_str(&serialized_str)?,
            &serde_json::from_str(&file_str)?,
        );

        if let Err(err_msg) = comparison_result {
            return Err(Error::new(err_msg));
        }
    }
    Ok(())
} */

#[macro_export]
macro_rules! parse_gdf_agent_files {
    ($name:ident, $type_deserialized:ty, $type_output:ty) => {
        fn $name(glob_exp: &PathBuf) -> Result<Vec<$type_output>> {
            let mut output_vec: Vec<$type_output> = vec![];
            let glob_str = glob_exp.as_path().to_str().unwrap();
            debug!(
                "parse_gdf_agent_files: entering parse_gdf_agent_files macro with glob_str {}",
                glob_str
            );
            for entry in glob(glob_str)? {
                let path = entry?;

                let file_name = path.as_path().to_str().unwrap();
                debug!("parse_gdf_agent_files: going to process file {}", file_name);

                // if not processing arrays (entity entries or intent utterances) skip
                // respective files (which are otherwise include in glob expresion)!
                if !glob_str.contains("_*.json")
                    && (file_name.contains("_entries_") || file_name.contains("_usersays_"))
                {
                    debug!(
                        "parse_gdf_agent_files: skipping the processing of the file file {}",
                        file_name
                    );
                    continue; // if not processing arrays (entity entries or intent utterances) skip respective files!
                }

                debug!("parse_gdf_agent_files: processing file {}", file_name);
                let file_str = fs::read_to_string(file_name)?;
                let deserialized_struct: $type_deserialized = serde_json::from_str(&file_str)?;

                let serialized_str = serde_json::to_string(&deserialized_struct).unwrap();
                let comparison_result = assert_json_eq_no_panic(
                    &serde_json::from_str(&serialized_str)?,
                    &serde_json::from_str(&file_str)?,
                );

                if let Err(err_msg) = comparison_result {
                    debug!(
                        "parse_gdf_agent_files: assert_json_eq_no_panic did not pass {}",
                        file_name
                    );
                    debug!("serialized_str {}", serialized_str);
                    debug!("deserialized_struct {:#?}", deserialized_struct);
                    return Err(Error::new(err_msg));
                }
                debug!("parse_gdf_agent_files: processed file {}", file_name);
                output_vec.push(<$type_output>::new(
                    file_name.to_string(),
                    deserialized_struct,
                ));
            }
            Ok(output_vec)
        }
    };
}

#[macro_export]
macro_rules! serialize_gdf_agent_section {
    ($iterator:expr, $folder:expr) => {
        for item in $iterator {
            let item_str = normalize_json_for_gdf_agent_serialization(
                &serde_json::to_string_pretty(&item.file_content)?,
            );
            let file_stem_option = Path::new(&item.file_name).file_stem();
            if let Some(file_stem) = file_stem_option {
                let mut file_handle = File::create($folder.join(format!(
                    "{}{}",
                    file_stem.to_str().unwrap(),
                    ".json"
                )))?;
                file_handle.write_all(item_str.as_bytes())?;
            } else {
                return Err(Error::new(format!(
                    "Unable to serialize file {}",
                    &item.file_name
                )));
            }
        }
    };
}
