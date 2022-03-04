from setuptools import setup, find_packages

setup(
    name='koko_keywords',
    version='0.0.2',
    author='Kareem Kouddous',
    author_email='api@kokocares.org',
    description="A python client  for the [Koko Keywords API](https://r.kokocares.org/koko_keywords/docs). The client handles caching to ensure very low latency.",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    url="https://github.com/kokocares/keywords-client",
    license='MIT',
    classifiers=[
      'Development Status :: 4 - Beta',
      'Intended Audience :: Developers',
      'Topic :: Software Development :: Build Tools',
      'License :: OSI Approved :: MIT License',
      'Programming Language :: Python :: 3',
      'Programming Language :: Python :: 3.6',
      'Programming Language :: Python :: 3.7',
      'Programming Language :: Python :: 3.8',
      'Programming Language :: Python :: 3.9',
    ],
    project_urls={
      'Documentation': 'https://github.com/kokocares/keywords-client',
      'Source': 'https://github.com/kokocares/keywords-client',
      'Tracker': 'https://github.com/kokocares/keywords-client/issues',
    },
    py_modules=['koko_keywords'],
    packages=find_packages(),
    include_package_data = True,
    install_requires=[
        'cffi==1.15.0'
    ],
    python_requires='>=3'
)