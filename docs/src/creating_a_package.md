# Creating an Evidence Package

When you first open EvidenceAngel, you will be greeted with a blank
screen:

![Nothing is Open in EvidenceAngel](./images/creating_a_package/0_nothing_is_open.png)

To create a new [_evidence package_](./glossary.html#evidence-package):

1. Select the main menu.

   ![main menu button](./images/creating_a_package/1_menu_button.png)
1. Select "New".

   ![the menu dropdown with "New" highlighted](./images/creating_a_package/2_menu_new.png)
1. Choose a file path to save your new _evidence package_. (Note: you
   have to choose a path when you create a file in EvidenceAngel in
   order to help prevent data loss in the event of a power cut)
1. You will now have an _evidence package_ open (notice the "Unnamed
   Evidence Package" at the top of the window), but no [_test
   case_](./glossary.md#test-case).

   ![no test case open](./images/creating_a_package/3_no_case_open.png)
1. Select the "Metadata" tab.

   ![metadata tab highlighted](./images/creating_a_package/4_metadata.png)
1. From here, you can rename the package, add a description, and add
   authors with the "+" button near "Package Authors".

   ![add author](./images/creating_a_package/5_package_metadata.png)
1. You can now continue by [creating a _test case_](./creating_a_test_case.md).

## Custom Metadata Fields

> **TIP**: Come back to this section once you've started getting used to
> EvidenceAngel. This is a more advanced feature!

You can create extra fields for test case metadata. These are created
per package and are set per test case. You can use them to keep track of
additional data, for example who is responsible of managing a particular
test case, or a particular environment you are working in.

When you create a new custom metadata field, you'll be asked for a name,
description, and internal ID. You can leave the internal ID blank if
you'd like, it'll be randomly generated internally, however it is often
worth giving it a short name, maybe something like `test-case-owner` or
`EnvironmentDetails`. It must be unique.

Optionally, one of the custom fields can be marked as "primary". This
will make the value set for this field in each test case appear under
it's name in the left navigation.

![various buttons associated with custom metadata](./images/creating_a_package/6_custom_metadata.png)
